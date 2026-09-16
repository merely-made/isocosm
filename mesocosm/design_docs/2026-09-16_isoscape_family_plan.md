# isoscape family plan

**Date:** 2026-09-16

**Status, 2026-09-16:** assessment complete, rulings 1 to 14 recorded; Phase D
(deep time) is next, orchestrated through subagents. No crate founded, no
code moved yet. The glyph expression plan's experience slice waits on Phase
D, by Mark's ruling of the same day
([glyph expression plan](2026-09-15_glyph_expression_plan.md), rulings on the
feat rule and pre-history).

**Owns:** the line between the three generation buckets Mark framed on
2026-09-16 (terrain in isometer, world in isoscape, history in the
hagiograph); the wing's world generation as a crate family named
**isoscape**; the map of the three worldgens that already exist; the wing
side of **deep time**, meaning what each product hands it and what its
continued clock breaks; and the order in which members are founded or
extracted.

**Does not own:** the hagiograph's own implementation, which lives in mere's
eidetic core (ruling 9) and needs its plan there; any product's simulation,
verbs or world truth. Isoscape
generates and hands over; the product's single authority runs the world from
then on (place-graph plan ruling 1). Nor the scene and terrain rendering
(isometer), the wing GUI (isomere), the glyph organ (hagioglyph), the
association organ (impresa), or a portable world-noun profile, which is
extracted after two real consumers and never declared in advance
(`mesocosm/CLAUDE.md`, Important Don'ts).

---

## 0. Rulings this plan rests on

**Ruled 2026-09-16 (Mark).**

1. **Worldgen belongs in isoscape.** The name was banked for worldgen on
   2026-09-15 (naming ledger): free on crates.io, a real geoscience term for
   an isotope landscape map, unclaimed until a real publish.
2. **The family is planned before anything is founded.** This document is
   that plan; no member exists until it is ruled.
3. **History does not start when you step in.** "The world's event log should
   be prefilled with basic events to overcome, due to worldgen." A generated
   world arrives with a past whose record a lineage has to beat.
4. **That past is simulated deep time**: the generated world's own simulation
   runs before handover, rather than marks derived by formula or a baseline
   table authored into the rules.
5. **Deep time's span is fixed in the world rules** and is part of the rules
   digest, so saves and replays cite it.
6. **The clock continues from deep time.** A played world starts at the tick
   and epoch deep time ended on, with one log that is literally history.
7. **The player enters as heir to a line that lived through deep time.** If
   that line died out, the draft offers a surviving one.
8. **The hagiograph is the history organ** (§3.0). It judges significance,
   through the record of standing marks and the feat rule over it; it
   generates the past that significance is judged against, through deep time
   and generated chronicles; and later it promotes, retells and memorializes.
   Storage of what happened stays in `muniment` journals and each product's
   own log, so the reservation's "not ordinary event history" boundary holds
   for storage. **Deep time is therefore the hagiograph's**, not an isoscape
   member, correcting this plan's first draft.
9. **The hagiograph stays in mere's portable eidetic core**, beside `muniment`
   and `chartulary`. The hagioglyph then consumes a mere crate, as Mesocosm's
   core already takes `muniment` and `nisus`.
10. **The record's mechanism moves to the hagiograph and its vocabulary
    stays with each product.** The mark lattice, holders, merge and the feat
    rule take product-defined axes; Mesocosm keeps its six feats and three
    scales as its axis set, and Paredros and the tabletop define their own.
11. **Terrain models live beside isometer's seam; isoscape owns the seeded
    pipeline.** The relief and brick description move into isometer next to
    the `Terrain` seam they fill, and isoscape owns the pipeline and the
    per-vessel presets that call them (§3.1).
12. **The genesis prefix is a wing rule.** Generated world history is
    accepted events on each vessel's one clock, and each vessel carries a
    test that a generated world record cannot be told from a played one.
13. **Replay keeps the handed-over baseline**: the world and its history,
    as the trial's baseline already does, rather than re-running deep time
    on load. Bytes grow with the prefix, bounded by compaction under G7's
    consequential-records rule.
14. **Deep time goes first, with a bench readout** of each boundary's
    population and feats. The relief lab opens the terrain and world lane
    afterwards; retuning relief or soil then means re-measuring the span.
15. **The heir is a choice, not one path** (answering R1, tentative in
    Mark's words: "All three are valuable. Probably looks like: Inhabit
    critter | Create new critter -> Align lineage with new critter?"). At
    entry the player either **inhabits a living critter** of a surviving
    line, or **creates a new critter** within the line deep time evolved;
    a created critter may then **align its lineage with itself**, which is
    today's redefinition of the line's program made an explicit act rather
    than a side effect of entering.
16. **Six epochs for now, and the span must vary** (R2: "This is gonna need
    to vary, so keep that in mind"). The span is a world rule, never a
    constant; six is only the generated preset's current value.
17. **The first slice carries deep time's history whole** and receipts its
    size per seed (R3). Compaction gets its own ruling once measured.
18. **Both products' terminology is updated now** (R4) to the history organ,
    the three generation buckets and deep time.
19. **Everything runs far during deep time.** With no hand there is no
    focus; frozen tiers would let the bodies near the released founder, and
    their descendants, run a different ecology from everyone else (§2.5).

**Standing rulings this plan inherits, not restates.**

- **Place-graph plan §0** (`2026-08-05_place_graph_engine_plan.md:14-81`):
  one authority per capability; shared organs are encouraged while they stay
  verb-neutral; **worldgen is hybrid, a top-down skeleton (relief,
  watersheds) and then bottom-up detail within regions** (ruling 7); world
  graphs must differ, receipted with graph metrics (ruling 5).
- **Place-graph plan §4** (`:555-562`): a shared organ serves three vessels
  or is Mesocosm-local and says so; extraction follows two real consumers.
- **Founding record** (`2026-07-30_games_wing_founding.md:142-147`): world
  generation produces working transmission paths and local variants; player
  histories displace generated history; playing the earlier games is never
  required.
- **Phenotype contract C5** (`2026-07-31_wing_phenotype_contract_plan.md:662-671`):
  generated history cannot overwrite accepted events; importing a played
  history displaces a compatible generated slot; no civilization simulator.
- **Law C** (`mesocosm-core/src/chronicle.rs:36-45`): a generated creature
  has a history of the same shape as a played one, and nothing structural
  tells them apart. Deep time satisfies this by construction, since its
  history is the simulation's own.

---

## 1. The worldgens as they stand

### 1.1 Mesocosm

Verified 2026-09-16. All of it lives in `mesocosm-core`, whose manifest takes
`isometer-core`, `wing-functions`, `wing-glyphs`, `wing-formats`, `muniment`
and `nisus` (`mesocosm-core/Cargo.toml`, `[dependencies]`).

**The top-down skeleton**, from the place-graph plan's G0 and G1, both
complete 2026-08-08:

| Module | Lines | Owns |
| --- | --- | --- |
| `places/relief.rs` | 174 | Continental relief by diamond-square: an integer heightfield, deterministic from its own seed, "world truth's input rather than a rendering asset" |
| `places/grown.rs` | 446 | The place graph, adjacency derived from traversability over the relief, replacing the lattice `Places::scatter` asserted |
| `places.rs` | 372 | Places, scale and regions: where things happen |
| `places/bricks.rs` | 160 | Mesocosm's terrain description and burrow nests over `isometer_core::ground`'s brick container, which moved with the isometer family split |
| `places/soil.rs` | 404 | Matter per voxel column, TD6's closed cycle: total matter conserved |
| `places/near.rs` | 369 | Near-tier kinematics over brick truth. **Simulation, not generation** |

**Founding**, one door with five public constructors
(`world/genesis.rs:63,70,96,114,128`), all reaching the private
`World::found(seed, organism_count, palette, founding, ruleset)` (`:143`):
the rng streams, grown places and ground, soil seeding, the lineages, the
founding roster with two thirds producers (`PRODUCER_SHARE`, `:521`), and the
founders' `Born` events held as pending world state. `World::new(seed,
organism_count)` is the bare test constructor, with **233 call sites in 80
files** across five crates. `world/genesis/tg1.rs` (102) holds receipts for
authored declarations "at the worldgen boundary".

**The generation door**, `world/generation.rs` (503) and its children, 2,570
lines with tests: a versioned `Request` (`VERSION = 4`, "bump when seed
streams, admission, or founding interpretation change", `:13`) carrying seed,
variation, place, candidate and attempt counts, founder count (default 24),
a soil range and `SoilPattern` (`habitat.rs`), body criteria (body plan,
archetype, structure, role, proportions) and an optional fixed body. `prepare`
builds a **foundation world** with the drawn soil pattern (`:235-262`) and a
draft of admitted candidate lives; `Prepared::enter(index)` clones the
foundation, returns the provisional founder's matter to the soil, pays the
chosen body's founding material from the local patch and swaps it into
organism 0 of lineage 1 (`:432-477`). Candidates are admitted by a bounded
habitat trial (`generation/trial.rs`, 358).

So the generation door mixes two things the family has to separate: **world
generation** (habitat, soil pattern, the foundation) and **life generation**
(body plans, archetypes, proportions, admission), which is phenotype work.

**Consumers.** The generation door is read by about twenty files, all in
`mesocosm-genet` (bench generation controls, the creator and its habitat
panel, `generate-start`, `generation_content`, `played.rs`, `main.rs`) and
`mesocosm-runtime`. The relief and grown places already have a **second
consumer**: Paredros's client carves its room into a grown Mesocosm hillside
(`paredros-client/src/room.rs`, `residency.rs`, `probe.rs`,
`bin/d1_depth.rs`). §1.3 maps that side.

### 1.2 The tabletop

Verified 2026-09-16 by a read-only survey; paths are from the repository
root. **The founding record's "typed worldgen (W0–W5)" is real as typed data,
a sandboxed Lua host and a commit path. It is not procedural generation of
factions, places, characters, routes, laws or history.**

**The record.** `design_docs/archive_docs/2026-08-08/2026-07-09_worldbuilding_generation_plan.md`
landed W0 to W5 on 2026-07-11 (`:4-5`): visibility and the GM store (W0),
items (W1), the Lua generator interface with an entropy tape, locks and
fixtures (W2), generated local maps lowered to `MapDocument` (W3), the
`CampaignWorld` of factions, places, characters, routes, laws, history and
storylets (W4), and one pack generating an inspectable draft (W5). The
faction "world tick" is outside W0 to W5 and landed later as C7.

**The code.** `isometry-campaign` holds the proposal and storage types:
`generator.rs` (491: `GenValue`, `GenerationRecord`, a SplitMix64
`EntropyTape`), `world/types.rs` (252: `CampaignWorld`, `HistoryEvent`),
`world/draft.rs` (221), `map.rs` (258), `fact.rs` (78), and `faction.rs`
(459), whose `faction_turn` is **the only Rust-side generator** (`:104`).
`isometry-system` hosts the sandboxed Lua runtime (`sys/generator.rs`, 325;
piccolo, fuel, output and depth caps). `isonetry`'s host commits proposals
(`session/host.rs:176-496`). Views and genet own the preview UI and
orchestration.

**The contract, which is the valuable part.** "A generator returns a
proposal, never a session mutation. The host validates and chooses which
proposal to commit as ordinary public events" (`generator.rs:3-5`). Replay
is commit-result, not re-execution: a `GenerationRecord` keeps the request,
its one entropy draw and the decoded proposal, "so a peer or restored
campaign never needs to rerun pack code" (`:190-194`). A committed id cannot
be overwritten (`world/ops.rs:69-80`), which meets contract C5.

**What is actually generated.**

- **Little is random.** `campaign.lua` never reads its entropy; two other
  packs use it for a name suffix; the watchtower varies about two bits
  (`ruined_tower.lua:6-23`). Shipped worlds are hand-written JSON in Lua.
- **No procedural terrain.** Maps are authored or sparse pack cells over
  default ground; the overmap atlas says of itself "this adapter does not
  manufacture Voronoi geography" (`isometry-views/src/overmap/atlas.rs:1-6`).
  `MapTerrain` fills the isometer seam from an authored height field at five
  voxels per tile (`scene/terrain.rs:69-72,198`).
- **Generated history is hand-written** with fixed negative times
  (`campaign.lua:27-30`, `ruined_tower.lua:54-57`), spread over four carriers:
  `CampaignWorld.history`, `WorldFact` journal entries, the
  `muniment::Journal<GameEvent>`, and chronicle deeds.
- **The faction tick is half-built**: banked time is spent but never accrued,
  and `radiant_quests()` has no caller outside its test.

**Against the wing's laws.** Law C is proven for Mesocosm creatures
(`isometry-campaign/tests/proof_pair.rs`) and **not for world history**: the
public `Generation` record, `faction-turn` ids and negative times all reveal
origin. `Arrival::record` clamps negative time to zero (`chronicle.rs:171`),
so pre-history is lost on the way into a chronicle, which meets ruling 6's
continued clock head on. The tape is not saved, the default seed is the wall
clock, and records carry no pack hash or version.

**Two findings for next door, not this plan's to fix.** `isometry-campaign`
is licensed MIT OR Apache-2.0 (`Cargo.toml:37`) while depending on MPL-2.0
wing crates and containing an MPL-2.0 file (`construction.rs:1-2`). And the
tabletop's consolidation plan assigns it "generator hosting, system plugins
and campaign proposal types" (`design_docs/2026-09-09_games_wing_consolidation_plan.md:49-51`).

### 1.3 Paredros

Verified 2026-09-16. `paredros-world` (11,508 lines in all) takes
`mesocosm-core`, `isometer-core`, `wing-functions`, `wing-glyphs`, `conatus`
and `paredros-identity` (`paredros-world/Cargo.toml`, `[dependencies]`). Its
generation is four calls, in order, and the first two are not Paredros's:

1. **`Places::grown(seed, side, extent)`**, Mesocosm's grown place graph over
   Mesocosm's relief (`paredros-world/src/world.rs:119`, importing
   `mesocosm_core::places::{Grown, Places}` at `:9`).
2. **`Ground::grow(&grown, extent)`**, isometer-core's brick container filled
   through the `Terrain` seam, which `Grown` implements in Mesocosm
   (`mesocosm-core/src/places/bricks.rs:56`).
3. **`WorldMap::generate(&grown)`**, Paredros's own structural addresses:
   surface and underground slots per place, whose occupants may change
   without changing containment or routes (`sites.rs:72-78`, 211 lines).
4. **`Population::generate(&world, config)`**, deterministic origins and
   genesis facts for named lives: site residents and migrations along
   routes, each life with a home slot, a body seed and a generated name
   (`population.rs:80`, 245 lines).

`World::generate` stamps `GENERATOR_VERSION = 1` into saves and refuses a
save from another version (`world.rs:15,236,254`), and hashes the grown
graph, ground and map as the base (`:136`). The autonomous simulation and
its regrow-and-replay persistence (`simulation.rs`, 471;
`simulation_record.rs`, 177) run the world afterwards and are not generation.

**So the top-down skeleton already has two consumers**, Mesocosm and
Paredros, reaching it through `mesocosm-core`, which the Paredros guidance
calls "current shared-organ evidence, not settled permanent ownership"
(`paredros/CLAUDE.md`).

### 1.4 Where the three already meet

**The `Terrain` seam in `isometer-core`** (`ground.rs:141`) is the one piece
of world generation all three products already share. Its module doc states
the division: "a product's terrain model decides every column's surface and
describes the voids under it; what lives here is the container those answers
fill" (`ground.rs:1-7`). Two production implementations exist:

| Implementation | Where | Serves |
| --- | --- | --- |
| `Terrain for Grown` | `mesocosm-core/src/places/bricks.rs:56` | Mesocosm, and Paredros through it |
| `Terrain for MapTerrain` | `crates/isometry-views/src/scene/terrain.rs:198` | the tabletop's authored maps on the shared scene |

The rest are test fixtures (`Flat`, `Bands`, `Fixture`).

---

## 2. Deep time

**Home: the hagiograph** (ruling 8), in mere. This section keeps the wing's
side: the measurements that sized it, what each product's continued clock
breaks, and the contract a product meets. The seam and span policy belong to
the hagiograph's own plan.

### 2.1 Measured

**Instrument:** `mesocosm-core/examples/deep_time_probe.rs` (committed
`52eccd9`), release build, the played critter idling so its instincts drive
it. At each epoch boundary it records the living count, the living species,
the readings, how many took the record, how many **beat a mark that stood
before that reckoning** (the feat rule ruled 2026-09-16), and the epoch's
wall time. It drives `World::apply` directly, with no runtime, checkpoints or
flow windows, which is also how deep time itself would run.

**On real generation foundations**, 24 founders, candidate 0 entered:

| Seed | Epoch | Living | Species | Readings | Beat a mark | Seconds |
| --- | --- | --- | --- | --- | --- | --- |
| 7 | 1 | 40 | 2 | 9 | 0 | 1.7 |
| 7 | 2 | 93 | 2 | 9 | 4 | 2.3 |
| 7 | 3 | 202 | 2 | 9 | 2 | 2.6 |
| 7 | 4 | 404 | 2 | 9 | 5 | 4.7 |
| 7 | 5 | 708 | 2 | 9 | 5 | 12.6 |
| 7 | 6 | 970 | 2 | 9 | 1 | 14.5 |
| 7 | 7 | 907 | 2 | 9 | 1 | 15.3 |
| 7 | 8 | 621 | 2 | 9 | 0 | 14.0 |
| 1 | 1 | 36 | 2 | 9 | 0 | 0.9 |
| 1 | 2 | 80 | 2 | 9 | 3 | 1.2 |
| 1 | 3 | 158 | 2 | 9 | 4 | 1.9 |
| 1 | 4 | 300 | 2 | 9 | 2 | 3.3 |
| 1 | 5 | 465 | 2 | 9 | 5 | 6.0 |
| 1 | 6 | 649 | 2 | 9 | 1 | 8.7 |
| 1 | 7 | 680 | 2 | 9 | 0 | 10.5 |
| 1 | 8 | 600 | 2 | 9 | 0 | 12.5 |
| 42 | 1 | 49 | 3 | 10 | 0 | 0.8 |
| 42 | 2 | 121 | 3 | 10 | 5 | 1.0 |
| 42 | 3 | 341 | 3 | 10 | 1 | 1.7 |
| 42 | 4 | 674 | 3 | 10 | 3 | 3.2 |
| 42 | 5 | 1,330 | 3 | 10 | 3 | 6.5 |
| 42 | 6 | 1,562 | 3 | 10 | 1 | 9.6 |
| 42 | 7 | 1,333 | 3 | 10 | 1 | 10.1 |
| 42 | 8 | 1,100 | 3 | 10 | 1 | 9.4 |

History after eight epochs: 47,675 entries (seed 7), 49,902 (seed 1),
313,570 (seed 42). Six epochs cost 38.4, 22.0 and 22.8 seconds.

**On the bare constructor**, `World::new(seed, 60)`, the same shape and a
harsher end: seed 7 peaks at 842 in epoch 5 and falls to 203 by epoch 8,
losing a species; seed 1 peaks at 866 and holds 493; **seed 42 collapses from
789 to 14 organisms and from 6 species to 1** between epochs 5 and 8. History
reaches 265,102 to 378,061 entries. At the first boundary of every run, 10 to
14 readings took an empty record and none beat a mark.

**What the numbers say.**

- **Every foundation booms**, from 24 founders to 650-1,560 living by epoch 6
  or 7, and then turns down. No lineage died out on a foundation within eight
  epochs.
- **Feats are common during the boom and rare after it.** Through epoch 5,
  1 to 5 of 9 or 10 readings beat a standing mark at each boundary; from
  epoch 6, 0 or 1. A span of about six epochs is where the record stops
  falling constantly on these seeds.
- **Cost grows with the population, not with the history.** Seed 42's bare
  world ran its eighth epoch in 1.6 seconds with 14 organisms and 378,061
  history entries behind it.
- **An unplayed enclosure is not stable.** Bare worlds crash after their
  peak, one almost to extinction. Deep time will hand over a world in some
  phase of boom and bust, and a fixed span decides which phase each seed is
  entered in.

### 2.2 What a continued clock breaks

Ruling 6 keeps one log across the handover. Measured against the tree, these
assume a world with no past:

- **`Trial::new` refuses any world past tick zero** (`runtime/trial.rs:110-111`),
  because "a later snapshot needs its runtime checkpoint/history too". The
  bench's world trial, the glyph experiment and every trial receipt go
  through it.
- **`Runtime::from_world` starts with an empty `History` and `epoch_seen: 0`**
  (`runtime.rs:193-213`). Handed a world in epoch 6, its first tick would see
  `world.epoch != seen` and reckon an epoch against an empty log
  (`reckon_if_ended`, `:131-136`).
- **`Runtime::replayed` rebuilds from `World::new(seed, organisms)`**
  (`runtime.rs:543-547`), so replay identity is seed, count and trace. It
  already cannot replay a generation-door world, which the trial handles by
  keeping the baseline world itself (`trial.rs:4-7`). A world with a past
  needs its deep-time history in that baseline, or a replay that re-runs deep
  time from the request and the span.
- **`Prepared::enter` swaps the chosen body into organism 0 of lineage 1**
  (`generation.rs:432-477`). After deep time, organism 0 may be long dead;
  ruling 7 makes the entry an heir of lineage 1 instead. The succession
  module already names heirs: "living descendants this world would let the
  player inhabit, eldest first" (`succession.rs:129-138`).
- **The creator says "Enter starts at tick zero"**
  (`mesocosm-genet/src/app/creator/reading.rs:195,262`).
- **Saves carry the world snapshot, and history lives beside it.** A world
  handed over after six epochs brings tens to hundreds of thousands of
  history entries, which a save either carries, compacts under the retention
  budgets G7 anticipates, or regenerates by re-running deep time.

### 2.3 Cost, and who pays it

Six epochs cost 22 to 38 seconds in a release build, and a debug world ticks
roughly fifty times slower (the ceiling commit's control test ran one
thousand ticks in 81 to 126 seconds). The span is world rules, so a test
world states a span of zero, and so does every existing `World::new` caller:
**deep time is a property of generated worlds, not of the bare constructor**,
and none of the 233 call sites pays it.

### 2.4 The contract

Deep time **runs a product's own simulation**; it never simulates anything
itself, so it adds no second authority (place-graph ruling 1). What a shared
member can own is the policy and the seam around that run. Illustrative, not
compile-ready:

```rust
/// The product's simulation, as deep time sees it.
pub trait Epochal {
    type Record;
    /// One tick of the world's own rules, with no hand on anything.
    fn advance(&mut self);
    /// Epochs this world has closed. Deep time stops on a count, never a clock.
    fn epochs(&self) -> u64;
    /// What the world has seen, as the feat rule reads it.
    fn record(&self) -> &Self::Record;
}

/// World rules, inside the digest (ruling 5).
pub struct DeepTime { pub epochs: u32 }
```

The handover, in order, with ruling 6's continued clock and ruling 7's heir:

1. **Generate the foundation** as today, with no played body chosen yet.
2. **Run `DeepTime.epochs` of the world's own epochs** with no hand: no
   control, no checkpoint, no answer to any question. Every event goes into
   the one history, every boundary reckons into the one record, exactly as a
   played run does, so Law C holds by construction.
3. **Stop on the boundary.** The world is handed over at the tick its last
   epoch closed, so the played world's first epoch begins on a fresh budget
   and its first reckoning is judged against a record deep time filled.
4. **Draft candidates against the handed-over world**, not the foundation:
   soil, occupancy and the living lineages have all moved.
5. **Enter as heir.** The chosen body enters as a member of lineage 1 if it
   survived, placed and paid for as `enter_admitted` does today; if it died
   out, the draft offers only surviving lineages. The succession module's
   heir reading is the precedent.
6. **The baseline is the handed-over world plus its history.** A runtime
   built from it starts with that history and `epoch_seen` equal to the
   world's epoch; the trial accepts a world with a past when its baseline
   carries the past; replay starts from the baseline (ruling 13).

Replay is settled by ruling 13. Two questions in that list remain, and §5
asks them: what the heir's body is paid from and what it is to its line, and
whether the first slice compacts the history it carries.

### 2.5 What deep time handed over (measured after D7a)

Through the generation door with a six-epoch span, 24 founders, release
build, reproduced by the orchestrator with `deep_time_probe ... door`:

| Seed | Living | Lineages alive | Candidates drafted | Seconds | History |
| --- | --- | --- | --- | --- | --- |
| 7 | 900 | 4 | 0 of 128 attempts | 22.8 | 51,432 |
| 1 | 599 | 2 | 0 of 128 attempts | 15.6 | 17,110 |
| 42 | 1,287 | 2 and 4 | 4 | 16.1 | 314,575 |

**The player's line, lineage 1, died out on all three seeds.** On seeds 7
and 1 every draft attempt was refused for "insufficient local founding
matter for body and reserve": the patch's matter is held in the living.

**Compare §2.1's runs**, the same foundations with the played critter idling
under a hand: seed 7 kept two lineages to epoch 8 and lineage 1 among them.
Two things differ with no hand, and the measurements do not yet separate
them. The released body and its line take their own adaptation turns. And
**with no focus the ecology skips its near/far tier updates**, so every body
stays at the tier it had when control was released. The second is a
simulation difference, not an ecological one, and could bias which lines
survive.

**Ruled 2026-09-16 (Mark): everything runs far during deep time** (ruling
19). Measured again with every body far, same door and span:

| Seed | Living | Lineages alive | Player's line | Candidates drafted | Seconds |
| --- | --- | --- | --- | --- | --- |
| 7 | 862 | 1 and 4 | survives | 4 | 28.3 |
| 1 | 624 | 2 | extinct | 0 of 128 attempts | 15.7 |
| 42 | 1,046 | 1, 2 and 4 | survives | 4 | 29.6 |

So the frozen tiers were changing who survived. Ruling 7 already answers an
extinct line: the draft offers a surviving one.

**Where the matter went.** Asked about founding matter, Mark asked back:
"Isn't the role of micro and myco to refresh the soil through death and
decomposition?" `matter_ledger_probe` reads the enclosure's matter at each
epoch (the chosen body entered first, then six epochs of deep time, every
body far):

| Seed | Soil at start | Soil at epoch 6 | Carrion at epoch 6 | Decomposers alive |
| --- | --- | --- | --- | --- |
| 7 | 2,145,253 mg | 135,980 mg | 644,337 mg in 203 bodies | 2 at start, 0 from epoch 1 |
| 1 | 1,894,285 mg | 221,688 mg | 453,998 mg in 167 bodies | 2 at start, 0 from epoch 1 |
| 42 | 2,253,059 mg | 1,132,267 mg | 321,747 mg in 535 bodies | 2 at start, 0 from epoch 1 |

**The decomposers starve in the first epoch on every seed**, frozen tiers or
far, and consumers go with them on seeds 7 and 1. Nothing returns the
carrion, so on seed 7 carrion holds 30% of all matter by epoch 6 and the soil
keeps 6%; the matter near the draft's place falls to 29-79 mg on seeds 7
and 1. The empty draft is a symptom of a broken soil cycle, not of crowding.

**This is a known open ruling, not something deep time introduced.** The
open rulings register's item 2 (`2026-08-29_open_rulings_register.md:63-76`)
records decomposers starving beside 12-15 standing corpses: "the binding
constraint is `DECOMPOSE_RANGE` and the search, not the yield", with no
options named, and quadrupling the decay yield did not rescue them. The
scale plan found decomposers surviving to the horizon in 6 of 10 seeds once
the enclosure gave them room (`2026-08-29_scale_plan.md:415-424`). The
generation door's foundation is 24 founders in the small habitat. Deep time
makes the failure decisive: six epochs without decomposers lock a fifth to
a third of the enclosure's matter in carcasses.

**Open before D7b's create path.** Whether decomposer reach is ruled and
fixed first, since creation needs soil and the soil needs decomposers;
inhabiting a living critter needs no founding matter and does not wait on it.

---

## 3. Family layout

### 3.0 Three buckets, as Mark framed them (2026-09-16)

Mark, thinking it through rather than ruling: "May make sense to make
isometer terrain gen, isoscape world gen, and hagiograph history gen. But
those buckets don't seem equally big." And then: "The prior split of
hagiograph for unusual history is meant to distinguish significant events
from less significant ones. But part of that scope was taken up by
hagioglyph. So I figure the hagiograph bucket is more open than we planned."

**What the hagiograph was scoped to**, across the record:

- **What memory keeps** when a life ends, and the attention mechanic Law B
  depends on (`2026-07-30_mesocosm_founding_plan.md:302-306`;
  `2026-07-30_games_wing_founding.md:1210-1212`).
- **Promoting unprecedented, legendary and narratively significant events**
  out of the timeline, for the stack's procedural voxel engine to manifest
  (`mesocosm/CLAUDE.md`, terminology; presentation plan L8,
  `2026-09-11_orthographic_voxel_presentation_plan.md:1397-1400`).
- **Retaining significant history** (`2026-08-06_general_model_plan.md:574`)
  and making legendary craft consequential rather than a rarity tier (`:766`).
- **A view over journals**, holding "the retold subset": legends, memorials,
  epithets and manifestations, whose presence scales with retelling. "Not
  ordinary event history. That is a `muniment::Journal`"
  (`mere/crates/eidetic/hagiograph/README.md`, a 26-line reservation).
- **Paredros's lane H**: retelling, remembrance, significance and
  manifestation proposals (`paredros/design_docs/2026-09-09_functional_loops_plan.md:54`).

**What has been taken from it since.** The **hagioglyph** took the divinity
half on 2026-09-15: the canon, the journey, ascension, **the chosen referent
and its periods**, and revisions, consuming the hagiograph one way
(`2026-08-06_general_model_plan.md:960-970`). **Impresa** took association
records and their fact, belief and legend readings, leaving the hagiograph
the promotion that makes an association legend (G7, `:997-1033`).

**What is left, and unclaimed.** Judging significance, and the past that
significance is judged against. Mesocosm already holds the lookup half of
that judgment in `record.rs` (405 lines), whose own docs name the split:
"the journal holds everything, tulpa holds what is retold" (`:43-44`, tulpa
being the hagiograph's earlier name). The feat rule ruled today, beat a mark
that stood before the reckoning, is a significance rule over that record.
Deep time is how a world comes to have a record at all.

**So the three buckets, with what each would hold today:**

**Ruled the same day** (rulings 8 to 10): the hagiograph takes the history
bucket, stays in mere, and receives the record's mechanism.

| Bucket | Would hold | Size today |
| --- | --- | --- |
| **isometer: terrain** | Terrain models that fill `isometer-core`'s `Terrain` seam: Mesocosm's relief and brick description, beside the container they already fill | about 330 lines moving; the tabletop adds none, since its terrain is authored (§1.2) |
| **isoscape: world** | The place graph, soil and habitat, founding rosters, Paredros's sites and population, and the tabletop's typed nouns (§1.2) | about 2,480 lines from Mesocosm and Paredros; the tabletop adds its proposal-and-commit contract and typed nouns, not generators (§1.2) |
| **hagiograph: history** | The significance record and its rules, deep time, generated chronicles, and later promotion, retelling and memorials | about 405 lines of record plus deep time; the tabletop's history is hand-written and its faction tick half-built (§1.2) |

**Unequal, and that is not the test.** A boundary follows a capability's
authority, not a balance of lines. Terrain is small and fully cohesive: a
seam and the models that fill it. History is small today and the most open to
growth, since promotion, retelling and memorials are all unbuilt. World is
the large one, and may itself need members.

**The joint to watch.** The place graph sits exactly on the terrain and world
line: its adjacency is derived from traversability over the relief
(`places/grown.rs:1-6`), and ruling 7's hybrid worldgen is that joint.

### 3.1 A second opinion, reconciled (2026-09-16)

Mark shared another agent's read. Where it was checked against the tree:

- **"Terrain generation is a 65-square relief field and about four hundred
  bricks, coarse by a deliberate 2026-08-07 ruling."** Holds: `SIDE: usize =
  65` (`places/relief.rs:20`), and the place-graph plan's 2026-08-07 entry
  calls authoritative generation "deliberately coarse (65² relief, ~400
  bricks, milliseconds)" (`:593-601`).
- **"The relief lab was adopted on 2026-08-07 and never built."** Holds: an
  instrument for live seed and parameter tuning over `grown` and `Ground`
  (`:593-601`), listed as "instrument, any time" in the dependency ledger
  (`2026-08-07_dependency_ledger.md:226`), with no code anywhere. The
  presentation-amplification tier is likewise adopted without a receipt.
- **"It is now the only world model behind three vessels."** Overstated.
  It is the only *procedural* world model, behind two: the tabletop's terrain
  is authored and reaches the same seam through its own adapter (§1.4).
- **"Cleromancy already picks an Isometry generator by divination."** Holds,
  and it picks which generator runs, never its output
  (`isometry-genet/src/cleromancy_selection.rs:1-6`).

**Where it agrees with today's rulings.** Nothing consolidates how the wing
generates (§1.4). Generation must not become a second authority. And its
first recommendation, a **genesis prefix** of accepted events replayable from
the seed rather than generated-provenance attributes, is what rulings 3, 4
and 6 already chose for Mesocosm: deep time's history is the simulation's own
accepted events on one clock.

**Where it predates them.** It puts relief, places, links, ground and the
prehistory into isoscape as one seeded pipeline. Since then the history
bucket went to the hagiograph (ruling 8), and terrain in isometer is framed
but not ruled. The two readings meet if **isoscape owns the seeded pipeline
and its per-vessel presets, calling terrain models that sit beside
isometer's seam and deep time in the hagiograph.**

**What it adds that this plan lacked.**

- **The prefix as a wing rule, not a Mesocosm one.** The tabletop already
  commits generated proposals as ordinary events, which is a prefix by
  commit-result rather than by re-execution. But its history still reveals
  origin (§1.2), so Law C is unproven for world history anywhere.
- **The relief lab as the instrument generation is tuned with**, now cheap
  because the bench and isometer supply the live viewport it waited on.
- **The cost of a long prefix** is history bytes, which makes G7's rule that
  consequential records never compact carry more weight than it did.

### 3.2 The layout, as ruled

| Family | Repository | Gains | First consumers |
| --- | --- | --- | --- |
| **hagiograph** | mere, `crates/eidetic/hagiograph` | The mark record over product-defined axes and holders, with merge and the feat rule; the deep-time seam, span and handover receipt; later promotion, retelling and memorials | Mesocosm, then Paredros |
| **isometer** | isometry, `shared/isometer` | Terrain models beside `isometer-core`'s `Terrain` seam: Mesocosm's diamond-square relief and brick description | Mesocosm and Paredros through isoscape; the tabletop keeps its authored adapter |
| **isoscape** | isometry, `shared/isoscape` | The seeded pipeline and per-vessel presets; the place graph extracted from `mesocosm-core`; world generation split from life generation | Mesocosm and Paredros, which already share the skeleton |

**Stays where it is.** Each product's journal and log; Mesocosm's feats,
scales, soil ledger and founding roster; Paredros's sites map and population;
the tabletop's Lua host, proposal types and commit path, which its
consolidation plan assigns to it; every product's simulation.

**Licenses.** isoscape and the moved terrain models are MPL-2.0 under the
wing's 2026-09-03 ruling; the hagiograph follows mere's workspace license.

## 4. Move order

Each step is green and revertible alone. Steps that wait on nothing, or on
the same finished step, can run in parallel.

### Phase D: deep time

| Step | Where | Waits on | Model |
| --- | --- | --- | --- |
| **D1.** Rescope the hagiograph reservation (README, manifest description) to the history organ, and open its dated plan in mere | mere docs | nothing | the orchestrator |
| **D2.** The mark record: `Mark<H>` and a record over axis `A` and holder `H`, with `standing`, `is_unprecedented`, `untouched`, `note`, `merge`, `axes`, `filled`, and the feat rule judged against the marks **as they stood before** a reckoning. Postcard bytes identical to Mesocosm's `WorldRecord` for `(Feat, Scale)` and `SpeciesId` | mere `hagiograph` | D1 | sonnet |
| **D3.** The deep-time seam: an `Epochal` trait, `DeepTime { epochs }`, and a driver that runs a simulation with no hand until its epoch count, stops on the boundary and returns a handover receipt. Tested on a toy simulation | mere `hagiograph` | D2's module layout | sonnet, with D2 |
| **D4.** The continued clock in the runtime: a runtime built from a world with a past carries its history and starts `epoch_seen` at the world's epoch; the trial accepts a baseline world with its history and replays it exactly; `Trial::new` keeps refusing a past with no history | `mesocosm-runtime` | nothing | sonnet |
| **D5.** The advance-to-boundary control, ruled 2026-09-16: one bench action steps the trial until the next epoch boundary, a checkpoint or the ceiling, with probe fields for the epoch and the last boundary tick | `mesocosm-genet` bench | nothing | sonnet |
| **D6.** Mesocosm reads the hagiograph's record: `WorldRecord` becomes Mesocosm's axis set over the moved mechanism, re-exported at its old paths; the six byte pins hold | `mesocosm-core` | D2 pushed and pinned | sonnet |
| **D7a.** Deep time in Mesocosm: `WorldRules` gains the span (bare worlds and requests zero, never a constant; **the world-state pin and the journey hashes re-pin once**, since postcard writes every rules field); `World` runs deep time with no hand through the hagiograph's seam; the generation door runs it on the foundation and drafts against the handed-over world, and refuses entry into a world with a past until D7b. **No `VERSION` bump**: a request without the span founds the same world. **Landed 2026-09-16** | `mesocosm-core` | D3, D4, D6 | opus |
| **D7b.** The heir's entry, ruling 15: **inhabit** a living critter of a surviving line; **create** a new critter within the evolved line, paid from local soil, leaving the line's program as deep time left it; and **align**, an explicit act that makes a created critter's recipe its line's program. The generated preset's span of six (ruling 16) turns on here, once entry works. **Blocked on §2.5's questions** | `mesocosm-core`, creator in genet | D7a, §2.5 | opus |
| **D8.** The baseline save carries the history whole, with a size receipt per seed (ruling 17) | `mesocosm-runtime`, genet | D7a | sonnet |
| **D9.** The bench readout: each deep-time boundary's population, species and feats, from the handover receipt | `mesocosm-genet` | D5, D7a | sonnet |
| **D10.** Law C for world history in Mesocosm: a generated world's events and marks cannot be told from a played run's | `mesocosm-core` tests | D7a | sonnet |

**Cross-repository pinning.** The Mesocosm workspace takes mere by one git
rev (`mesocosm/Cargo.toml:31-47`, rev `876320fd`). Bumping that rev pulls
every mere change since into five crates, so **the hagiograph is pinned alone
at its own rev** until a deliberate whole-mere bump. It depends on serde and
nothing else, so two revs of one repository coexist without a shared
dependency conflict.

**Then the experience slice resumes**, with its feat tier reading the
hagiograph's rule over a record deep time filled.

### Phase W: terrain and world

| Step | Where | Waits on |
| --- | --- | --- |
| **W1.** The relief lab: live seed and parameter tuning over relief, places and ground in the bench viewport, with the distinctness metrics G0 receipts | `mesocosm-genet` | Phase D |
| **W2.** Relief and brick description move beside the seam, re-exported at Mesocosm's old paths; Paredros unchanged | `shared/isometer` | W1 |
| **W3.** Found isoscape: the seeded pipeline and presets, and the place graph extracted from `mesocosm-core`; Mesocosm and Paredros both consume it | `shared/isoscape` | W2 |
| **W4.** Split the generation door's world generation from its life generation | `mesocosm-core`, isoscape | W3 |
| **W5.** Paredros meets ruling 12: its simulation as deep time and a Law C test for world history | Paredros | W3, D3 |
| **W6.** The tabletop meets ruling 12 in its own lane: origin-free generated history on one clock, and a Law C test | tabletop | its own plan |

## 5. Risks and decisions for Mark

**Asked before D7, answered 2026-09-16 as rulings 15 to 18.**

- **R1. The heir.** Found while asking: entering today also **overwrites
  lineage 1's program** with the candidate's recipe
  (`generation.rs:450-452`), which after deep time would erase what the line
  evolved. Answered by ruling 15, which makes that overwrite an explicit
  "align" act.
- **R2. The span.** Six for now, varying later (ruling 16).
- **R3. Compaction.** Carried whole first (ruling 17).
- **R4. Terminology.** Updated now (ruling 18).

**Still open.**

- **What varies the span.** Ruling 16 says it must vary and not how: per
  preset, per world condition, or per period. D7a keeps it a world rule so
  any of those can set it.
- **Inhabiting at entry and succession.** Inhabiting a living critter at
  entry resembles succession after a death; whether they share one door is
  D7b's to propose.

**Risks.**

- **Deep time hands over a boom or a bust.** A fixed span enters each seed in
  a different phase, and bare worlds crashed (§2.1).
- **Generation cost is paid at preview.** Drafting against the handed-over
  world means deep time runs before a candidate is shown, 22 to 38 seconds.
- **The record move must keep its bytes.** `WorldRecord` is serialized in
  the world snapshot (`world.rs:263`), so D6 is byte-pinned.
- **Two mere revs in one workspace** is deliberate and temporary, and the
  cargo resolution traps apply.
- **Next door, not this plan's:** the tabletop's MIT OR Apache-2.0 crate
  holding MPL-2.0 code, and `Arrival::record` clamping negative time (§1.2).

## 6. Done conditions

**Phase D is done when:**

1. The hagiograph's record passes its lattice laws (commutative,
   associative, idempotent merge), refuses a first mark as a feat, and judges
   two same-boundary beats of one older mark as two feats in either order.
2. Its postcard bytes for Mesocosm's axis set equal `WorldRecord`'s, and the
   six byte pins hold through D6.
3. A toy simulation under `DeepTime { epochs: 3 }` stops on its third
   boundary and returns a receipt naming it.
4. A runtime built from a world with a past does not reckon on its first
   tick, and a trial over that baseline replays to the same hash.
5. One bench action advances a trial to its next boundary, stated in probe
   fields, with the trial scenarios passing unweakened.
6. A generated world arrives past its span with its first played reckoning
   judged against a non-empty record; the pins re-pin once, in the commit
   that adds the span, with the reason stated; no code path treats six epochs
   as a constant.
6b. A player can enter a handed-over world by inhabiting a living critter or
   by creating one within the evolved line, and can align a created critter's
   line only through the explicit act, with the line's program otherwise
   unchanged from what deep time left.
7. A save carries the baseline and its history, with bytes receipted per
   seed.
8. The bench shows each deep-time boundary's population, species and feats.
9. Mesocosm's Law C test for world history passes.
10. The experience slice's feat tier reads the hagiograph's rule.

**Phase W is done when** the relief lab tunes a live world, the relief and
place graph live in isometer and isoscape with Mesocosm and Paredros
unchanged in behaviour and pinned bytes, and both simulating vessels pass a
world-history Law C test.

---

## Findings

- **2026-09-16.** `WorldRules` is serialized positionally with postcard in the
  world snapshot, so adding the deep-time span moves every world's bytes, as
  each earlier rules field did (`rules.rs:197-221`).
- **2026-09-16.** The tabletop generates almost nothing procedurally; its
  contribution to the wing is the proposal-and-commit contract (§1.2).
- **2026-09-16.** The relief lab, adopted 2026-08-07, has no code (§3.1).
- **2026-09-16.** Mesocosm's worldgen already has a second consumer in
  Paredros's client, which builds on the grown relief (§1.1).
- **2026-09-16.** The generation door mixes world generation with life
  generation (§1.1).
- **2026-09-16.** At a fresh world's first epoch boundary, 14 of 21 readings
  took the record because the record starts empty (measured in the glyph
  expression plan §9.8); deep time is what fills it.

## Progress

- **2026-09-16.** Plan opened. Rulings recorded, Mesocosm mapped, deep time
  measured on six worlds across three seeds.
- **2026-09-16.** Tabletop and Paredros mapped; the hagiograph ruled the
  history organ; a second opinion checked and reconciled; rulings 11 to 14;
  layout, move order and done conditions written.
- **2026-09-16.** Rulings 15 to 18. D1 landed in mere (`4a895c5e`: the
  hagiograph rescoped, its plan opened). D2 to D4 dispatched to Sonnet agents
  in parallel: the hagiograph's record, feat rule and deep-time seam in mere,
  and the runtime's continued clock in Mesocosm.
- **2026-09-16.** D2 and D3 landed in mere (`53648d3a`, pushed): `Record`,
  `Mark`, `reckon` with `took` and `feat`, and the `Epochal` seam with `run`;
  21 tests, clippy, rustfmt and a wasm32 check clean on rerun; the record
  reproduces `WorldRecord`'s postcard bytes. D4 landed in Mesocosm: a runtime
  built from a world with its history starts `epoch_seen` at the world's
  epoch, and `Trial::with_past` accepts a baseline with its history,
  including one standing on a boundary, while `Trial::new` keeps its
  refusals; replay code moved verbatim to `runtime/replay.rs` to stay under
  the line ceiling; 56 unit tests on rerun (79 with integration tests, as
  the agent ran them).
- **2026-09-16.** D5 landed (`cbf343c`): Advance to boundary steps a trial a
  hundred ticks a frame to its next boundary; probe fields `trial-epoch`,
  `trial-last-boundary-tick`, `trial-advancing`; `trial-boundary.scenario`
  reaches ticks 1,000 and 2,000 with no checkpoint. Review found the last
  boundary noted only during an advance run, fixed before commit. Spatial
  coverage byte-identical over 32 captures. The three saved comparisons in
  `receipts/2026-09-13/integrated-world` no longer load: their hashes predate
  the two `WorldRules` fields added that evening (`c2a26c2`, `2594031`).
- **2026-09-16.** D6 landed: `WorldRecord` is a transparent newtype over
  `hagiograph::Record<(Feat, Scale), SpeciesId>`, `Mark` a type alias, every
  method delegating with its signature unchanged; the hagiograph is pinned
  alone at mere `53648d3a`. Built in an isolated worktree; the six byte pins
  hold with no value edited, the core suite is 746 before and after, and in
  the main tree the pins, record tests, reckoning test and a Paredros
  workspace check all pass.
- **2026-09-16.** Ruling 19 landed: `World::run_deep_time` sets every
  living body far after releasing control (`freeze_tiers_far`), since no
  focus exists to update tiers and offspring inherit their parent's. Taking
  control afterwards makes bodies near again. 760 core tests; pins hold.
  `matter_ledger_probe` added. §2.5 records both measurements and the
  decomposer finding.
- **2026-09-16.** D7a landed. `DeepTimeSpan` joins `WorldRules` (digest and
  serde default zero); `World::run_deep_time` releases control through a
  crate-internal door that leaves `control_lost` alone, then drives
  `hagiograph::run`, with a ceiling of span plus one epochs of ticks and a
  named refusal for rules that never close an epoch; `Request.deep_time`
  makes `prepare` run it and keep the history on `Prepared`, and every
  entry door refuses with `HeirEntryNotBuilt` for a non-zero span. Only the
  world-state pin moved, `df397e7c183eec55` to `88662e9be82d7bef`, and
  removing the one new zero byte reproduces the old hash; journey hashes
  moved with it. Core 746 to 757 tests, runtime 79 to 80, all workspaces
  compile. The plan's row had said the generated preset six and a `VERSION`
  bump; the orchestrator's brief deferred the preset to D7b and kept
  `VERSION` 4, and the row now says so. §2.5 records what the handover
  produced.
