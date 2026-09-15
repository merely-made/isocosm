# Glyph expression plan

**Status: plan, 2026-09-15.** Assessment only; no code moved, nothing
committed. Supersedes the acquiring-act axis of the
[effect pack preset plan](2026-09-15_effect_pack_preset_plan.md) §3 and its
rulings 2, 5 and 6. That plan's shared declaration (`wing-glyphs/src/pack.rs`),
`mesocosm-core`'s execution table, the bench's journey binding, the read-only
invariant, the amount curve and the probe fields are kept and reworked here;
its steps 0 to 3 stay the record of what landed.

Implements the next move under general model
[§7.4](2026-08-06_general_model_plan.md#74-glyph-canon-the-journey-and-divinity-2026-09-13),
after G3's first slice.

## 0. Mark's ruling (2026-09-15)

> "Experiencing the glyph is the main thing. Depending on the sort of divinity
> you want to mantle, you will probably need different glyphs embodied. But you
> need to experience them all in some manner, and the conditions should be
> varied and trigger according to the log of significant events. It also makes
> sense to change what counts to trigger a glyph experience. But generally, for
> Mesocosm, we should first focus on expressing every glyph with a trait
> (likely multiple) and then we can go from there."

Earlier the same day: a glyph is **had**, not performed; a critter's access is
**embodied** (a trait or part bears it), a borg's may be **held** (an item,
sacrificable at ascension), a character's is wider still; **you cannot sacrifice
a carving**; **you must have the glyphs to ascend using them.**

Two consequences that shape everything below. The journey-rule axis is no longer
how the glyph was *earned by an act*; it is **what bears it**. And embodiment and
experience become two different facts about the same glyph, kept in two different
places: embodiment in the body, experience in the journey.

**The first slice is the ruled one:** express every glyph in a canon with one or
more traits. Everything in §3 is design until Mark rules it.

---

## 1. What a trait is, in Mesocosm's data

There is no type called `Trait` that means what Mark means. Six things in the
tree are candidates, and only one of them is rule-bearing, per-part and
addressable:

| Word | Type | Where | What it is |
| --- | --- | --- | --- |
| **part** | `Part` | `shared/isometer/crates/isometer-core/src/body.rs:138-162` | Geometry, mass, pivot, attachment, provenance, `severed` flag. Carries **no** capability field |
| **role** | `Role` | `isometer-core/src/plan.rs:82-92` | Four shapes, recomputed from `half_extent` by `classify` on every call. Not stored |
| **process** | `Process` (5 natives), `ProcessDef`, `ProcessRef` | `mesocosm-core/src/process.rs:64-90`, `:365-426`, `:337-339` | A definition: qualified id, the roles that may express it (`admits`), and whether a shape *grows* it (`Seeding`). Identity is a `DefinitionDigest`, not the enum |
| **expressed site** | `Site { id, process: ProcessRef, cells, cause }` | `mesocosm-core/src/phenotype/mosaic.rs:68-77` | **The trait, as the data actually stands.** A named definition occupying named cells of a named part, with `Expressed::Geometry` or `Expressed::Arranged { revision }` recording why |
| **guise** | `Organism.guise: Kingdom` | `mesocosm-core/src/organism.rs:175` | A *claim* about the body, heritable, deliberately allowed to disagree with `kingdom()` (`organism.rs:437`). Not a trait |
| **material** | `Stock` / `Material`, the mosaic's `scruple` | `mesocosm-core/src/phenotype/mosaic.rs:121`, `:240` | What the part is *made of* (nis), not what it does |

Two more that look like traits and are not. `Appendage`
(`axis.rs:86-99`, six variants) is a **lexicon word**: what a *line* may grow,
inherited, not expressed on any body — `Recipe::lexicon` at `axis.rs:247`.
`Candidate` (`discovery.rs:225-243`) is a **developmental availability**: a
proposal, not an expression. `FaunaTraits` (`organism/behavior.rs:57-62`) is a
derived reading over parts, recomputed per tick, stored nowhere.

**So a trait is an expressed site: `(PartId, ProcessRef)`.** The plain-English
test Mark's ruling needs — *a trait or part bears the glyph* — lands exactly on
that pair, and nothing else in the tree is both borne by a part and rule-bearing.

### How a critter comes to have one, and loses it

- **Founding anatomy.** `BodyPhenotype::seed` (`phenotype.rs:85-99`) makes one
  `Mosaic` per part and seeds it from the part's shape: `Role::processes`
  (`process.rs:130-144`) says a limb contracts, a mass takes in, a sensor senses,
  a plate fixes. Whole-part expression — the seeded site takes every cell
  (`mosaic.rs:124-130`).
- **Growth.** `Event::Grew { organism, part }` (`history.rs:97`) adds a part;
  seeding gives it its sites the same way.
- **Development.** `BodyPhenotype::develop` is the **one validator**
  (`phenotype.rs:33-41`), and the only way a site appears that geometry did not
  seed — `Process::Secrete` is the first definition no shape grows
  (`process.rs:79-89`), so expressing it takes tissue off something else and
  writes `Event::Expressed { organism, part, cost_mg }` (`history.rs:151-155`).
- **Grafting.** `phenotype/graft.rs` moves a branch with its mosaics and their
  `scruple` (`graft.rs:225`); `Event::Grafted` (`history.rs:163-172`) carries
  both subjects. **Inheritance**: `Event::Inherited` (`history.rs:198-204`), or
  `Event::Unexpressed` when the body cannot carry the line's revision.
- **Loss.** `Event::Severed { organism, part }` (`history.rs:104`) tombstones the
  part and everything below it; `Part.severed` (`body.rs:161`) is the flag and
  `BodyDocument::living` (`anatomy.rs:41-43`) filters on it. A severed mosaic
  **stays addressable** so an injury is explainable and is **excluded from
  `allocations()`** so it cannot contribute (`phenotype.rs:43-47`, `:111-117`).
  `Event::Died` (`history.rs:106-109`) ends the body.

### What already links trait to part and part to function

`BodyPhenotype::allocations()` (`phenotype.rs:111-117`) yields
`(PartId, &Mosaic)` for living parts only — trait to part, already index-aligned
and tombstone-safe. `ProcessDef::admits(role)` (`process.rs:423-425`) gates which
shapes may bear which definition. `mesocosm-core/src/functions.rs:16-23`
(`live_parts`) turns living parts into `wing_functions::PartRef`, and
`generate_for_body` binds a functional network to `BodySite { part, role }`
(`wing-functions/src/generation.rs:29-32`) — part to function, already
product-neutral and already refusing a site that is not live.

**One trap to name.** `BodyDocument::processes` (`process.rs:452-457`) answers
from geometry (`classify(half_extent).processes()`), not from allocation. Two
readings of "what this part does" exist, and only the phenotype's is the record
of expression. Embodiment must read `BodyPhenotype`, never `BodyDocument`.

---

## 2. Expression: every base glyph is expressed by one or more traits

### 2.1 The data shape

**A per-canon-revision expression table, in `shared/wing-glyphs`, beside
`pack.rs`, keyed by an opaque namespaced trait id.** Recommended, with the
reasons:

- **The precedent is Mark's own, days old.** `pack.rs` holds the declaration as
  product-neutral data and leaves execution product-side (`wing-glyphs/src/pack.rs`,
  `lib.rs:18-21`). An expression table keyed by trait id is the same object.
- **The key already matches on both sides.** `ProcessId::qualified()` returns
  `namespace:name` (`process.rs:304-307`); wing-glyphs' `identifier()` accepts
  exactly `namespace:local` with the same character class (`lib.rs:32-44`).
  Nothing is invented or converted.
- **The siblings need it.** Paredros and Isometry have their own trait
  vocabularies and divinities. A table keyed by `mesocosm-core::ProcessRef`
  could never be read by either; a table of strings can.
- **The edge is already paid.** `mesocosm-core` depends on `wing-glyphs`
  (ruling D1, landed).

What stays in `mesocosm-core` is the **resolution**: trait id → `ProcessRef`
through `Registry`, plus the reading over `BodyPhenotype`. Only Mesocosm knows
what a part is, so that half cannot be shared.

```rust
// shared/wing-glyphs/src/expression.rs — illustrative, not compile-ready.
// Data only. Nothing here reads a body, draws, or grants.

/// An opaque product trait identity, `namespace:local`. The kernel never
/// resolves it; a product maps it to its own bearer type.
pub type TraitId = String;

/// One base glyph and the traits that express it, in this canon revision.
pub struct GlyphExpression { pub glyph: GlyphId, pub traits: Vec<TraitId> }

pub struct ExpressionSpec {
    pub version: u32, pub id: String,
    /// Must equal the canon revision this table was authored against.
    pub canon_revision: u64,
    pub entries: Vec<GlyphExpression>, pub limits: ExpressionLimits,
}
/// Arc<ExpressionSpec> plus two BTreeMap indices, exactly as Canon is built.
pub struct ExpressionTable { /* .. */ }

impl ExpressionTable {
    pub fn new(spec: ExpressionSpec) -> Result<Self, String>;
    /// Empty is a refusal at `new`, never a silent answer here.
    pub fn traits_of(&self, glyph: &str) -> &[TraitId];
    /// The inverse. Many-to-many, both directions indexed.
    pub fn glyphs_of(&self, trait_id: &str) -> &[GlyphId];
    /// Every base in `canon` has a trait, and the revisions agree. Pure
    /// validation, mirroring `EffectPack::covers`; it expresses nothing.
    pub fn covers(&self, canon: &Canon) -> Result<(), String>;
    pub fn to_json(&self) -> Result<String, String>;
    pub fn from_json(json: &str) -> Result<Self, String>;
}
```

### 2.2 The rule for many-to-many

- **Multiple traits per glyph: disjunction.** Embodied when **any** expressing
  trait is expressed by a living attached part. Conjunction is deliberately not
  admitted in this slice: with five native definitions it would make most glyphs
  unembodiable, and a conjunction is an authored claim needing its own
  vocabulary (§7.4's rule against inferred judgments).
- **Multiple glyphs per trait: allowed and expected** (risk R2). `glyphs_of` is
  indexed for exactly that.
- **Bases only.** Variants resolve through `Canon::base_id` (`canon.rs:154-163`)
  before any lookup, as `GlyphReading::effect_bases` already does
  (`runtime/src/glyphs.rs:146-155`).
- **Revision-keyed.** One table per canon revision. §7.4's G5 rule stands: the
  live reading follows the **current** revision, completion is judged against
  the **founding** one (`journey.rs:231-236`, `:249-260`).

### 2.3 What "a critter embodies a glyph" means at runtime

> The set of glyphs a body currently embodies is a **pure function of its
> living, attached parts and their expressed sites.** No stored field, no
> event, no journey.

```rust
// crates/mesocosm-core/src/embodiment.rs — illustrative.
pub fn embodied(p: &BodyPhenotype, r: &Registry, t: &ExpressionTable)
    -> BTreeSet<GlyphId>
```

It walks `phenotype.allocations()` (living parts only), takes each `Site.process`
(a `ProcessRef`), resolves it through `Registry` to a `ProcessId`, qualifies it,
and unions `table.glyphs_of(...)`. `BTreeSet` for the core's ordering rule
(`lib.rs:19-21`). `&`-only; no world state.

**The body events, each a test:**

| Event | Effect on the embodied set |
| --- | --- |
| tick zero | Whatever founding anatomy seeded. The trial's bound organism is a producer (effect pack plan ruling 5) whose parts seed `Intake` on `Role::Mass` and `Fix` on `Role::Plate` (`process.rs:134-143`) — so it embodies the glyphs those two express, from the first frame, having done nothing |
| `Grew` | Monotone increase or no change. A new part seeds its sites |
| `Grafted` | Monotone increase or no change on the recipient; possible decrease on the donor, since the branch left it |
| `Severed` | Possible decrease. The tombstoned part drops out of `allocations()` (`phenotype.rs:111-117`) while its mosaic stays readable for the injury record |
| `Died` | The body stops being a bearer. The journey is untouched — that is §3's whole point |
| `Expressed` | Possible increase: a development put a site geometry never seeds |

The asymmetry against the old model is the visible one. Under the acquiring-act
axis the specimen painted nothing until its first accepted act. Under expression
it embodies from tick zero and **loses** glyphs when it is cut.

---

## 3. Experience: how an individual comes to have experienced a glyph

**Design, not ruled**, except where marked. What Mark ruled: experience is the
main thing, conditions are varied, they trigger off the log of significant
events, and what counts can change. Below is the smallest honest version of that.

### 3.1 Embodiment is not experience

| | Embodiment | Experience |
| --- | --- | --- |
| Lives in | the body — a pure reading over living parts (§2.3) | the journey — `Journey::grant` writes an `Acquisition` (`journey.rs:48-61`, `:237-248`) |
| Survives death | no | yes. `ascension_basis()` retains the grant prefix at first ascension (`journey.rs:397`) |
| Can be lost | yes, by severing | no. Reacquisition never rewrites the first acquisition (§7.4) |
| Gates | **mantling** — which divinity you may take | **ascension** — completion over the founding canon (`eligibility()`, `journey.rs:313-322`) |

This is Mark's "a glyph is had, not performed" and "you cannot sacrifice a
carving" in one table: a borg's *held* access is an item and can be given up; an
**embodied** access is a fact about a body and cannot be, and an **experienced**
glyph is a fact about the continuing individual that nothing can take back.

### 3.2 The provenance kind stops being degenerate

The superseded plan found that axis flat: every Mesocosm grant is
`ProvenanceKind::Event` (`runtime/src/glyphs.rs:251`), so `Journey::motif()`
(`journey.rs:381-394`) returns one group for any Mesocosm journey. Expression
changes that without inventing anything — a glyph experienced through the trait
bearing it is `ProvenanceKind::Trait`, a variant that has existed since G1
(`journey.rs:12-21`) and has never been written. The motif becomes non-trivial
the moment a second kind is written, which is the first real step toward
"different divinities from the same collection".

### 3.3 The condition vocabulary, at its smallest useful size

Mesocosm already has the shape to copy: `discovery::Condition` — declared input
lanes, one bounded rule, a content-addressed `ConditionId`, a recorded match
(`discovery.rs:96-215`, `discovery/conditions.rs:80-115`). An experience
condition is the same object with a different subject, not a second mechanism.
Three rules, each satisfiable by facts the tree already records:

1. **Borne** — the body has borne an expressing trait for N consecutive ticks.
   Needs an authoritative bounded accumulator, as `World::hunger_run` is for
   `Stress::Hunger`; a view-only trend cannot unlock anything (traits brief,
   2026-09-01 boundary).
2. **Used** — an expressing trait took part in a recorded flow: `RecordedFlow`
   under a `Process` the trait expresses (`flow.rs:231`). `absorb_uptake`
   (`runtime/src/glyphs.rs:202-234`) already reads flows and grants on them.
3. **Befallen** — a significant event names a part bearing an expressing trait:
   `Severed`, `Grew`, `Expressed`, `Grafted` (`history.rs:97`, `:104`, `:151`,
   `:163`) all carry a `PartId`. The trait is on the part; the glyph is in the
   table.

Each condition **declares its lanes**, and evidence of another lane never reaches
its rule, exactly as `Input`/`Condition::declares` does (`discovery.rs:76-92`,
`conditions.rs:112-115`). "What counts can change" is then a data change: the
table is per canon revision, and a revision is already a recorded, non-rewriting
world fact (§7.4 G5).

### 3.4 Where the hagiograph comes in

The hagiograph promotes unprecedented, legendary and narratively significant
events out of the timeline (repo `CLAUDE.md`). The "unprecedented" machinery
exists — `WorldRecord::note` and `is_unprecedented` over `(Feat, Scale)`
high-water marks joined by max (`record.rs:65-78`, `:93-97`, `:166-169`) — but
**it is not wired**: `Runtime::end_epoch` has one caller and it is a unit test
(traits brief §2), so `world.epoch` never leaves 0 and `WorldRecord` stays empty
in a real run. Condition rule 3's promoted-event source is therefore blocked
behind the epoch boundary, not behind this plan. Proposed reading: a promoted
event is an *ordinary* event that additionally carries a promotion receipt, and a
condition may require the receipt. It never becomes a second event log.

### 3.5 Completion against mantling

- **Experienced all → ascension.** Already built: `Journey::eligibility` requires
  the founding canon's every base (`journey.rs:313-322`) and `ascension_basis()`
  freezes the prefix.
- **Embodied the subset a divinity requires → mantling.** Proposed as a separate
  authored predicate over the §2.3 reading: a divinity declares required glyphs;
  you mantle it while your body embodies them. Mark's "you must have the glyphs
  to ascend using them" reads as the conjunction — experienced to ascend at all,
  embodied to ascend *using* those glyphs.
- Nothing here makes mantling durable. §7.4's non-scope stands: no new field in
  durable `World`, no playable reincarnation.

---

## 4. What the bench shows

**An embodied glyph's marks emit from the actual faces of the parts that express
it.** `BodyLayer::glyph_anchors` (`shared/isometer/src/anchors.rs:35-44`) already
returns the largest meshed face per living part, ordered and filtered by
`PartId`, with `centre`, orthonormal `right`/`up`/`normal` and `extent`
(`anchors.rs:19-31`). A `GlyphAnchor` is exactly the frame a `WorldPlane`
orientation wants, and `journey::glyph`'s `SurfaceInscription` arm already builds
one from hardcoded axes (`bench/trial/journey.rs:126-137`). So a mark can sit on
the very part whose site expresses the glyph: expression becomes visible on the
body rather than inferred from a receipt.

- **Experienced but not embodied** — in the journey, with no living part
  expressing it (severed, or a new body). A camera-facing mark at the body
  centroid, never anchored to a face, because there is no face to anchor to. The
  absence of an anchor *is* the distinction; no extra colour rule is needed.
- **Embodied but not experienced** — the ordinary state at tick zero. Draws
  nothing; the withheld count reports it. This is where the superseded plan's
  "paints nothing until acquired" survives.
- **The read-only direction is unchanged and now stronger.** `resolve` takes
  `owned: bool` by value and holds no `&mut` (`effect_pack.rs:266-278`); the
  embodied reading is `&`-only over `BodyPhenotype`. Nothing drawn can grant,
  and nothing drawn can express.

**Of the effect pack table, what survives.** `MarkRequest`, `MarkForm`,
`Amount`'s saturating integer curve, `Refusal`, `PackRule.stroke`/`color`/
`cost`/`citation`, `validate_against` and the `section::stroke()` seam, all
untouched. What changes is the key: `PackRule.acquired_by: Acquiring` becomes
`PackRule.borne_by: Bearer` — **what bears the glyph**, not which act earned it:

| Bearer | Form | Placement |
| --- | --- | --- |
| `Trait { id }` — a living part expresses it | `SurfaceInscription` | the `GlyphAnchor` of the part bearing it |
| `Journeyed` — experienced, not currently embodied | `SustainedEmission` | camera-facing at the body centroid |
| `Held` — an item (borg tier) | reserved, not implemented in this slice | — |

`Acquiring` is retired with §3 of the superseded plan, and `MarkRequest` gains
an `anchor: Option<PartId>` so the bench can pick the face without core naming a
renderer type.

---

## 5. What survives of the uncommitted work, file by file

| File | Verdict | Reason |
| --- | --- | --- |
| `wing-glyphs/src/pack.rs` (+ `pack_tests.rs`) | **Keep, untouched** | Declaration is orthogonal to expression. 204 lines of pure data, 5 tests |
| `wing-glyphs/src/expression.rs` | **New** | §2.1 |
| `mesocosm-core/src/effect_pack.rs` | **Rework** | Keep `MarkRequest`, `MarkForm`, `Amount`, `Refusal`, `validate_against`, the curve constants, the revision-pinned lookup. Replace `Acquiring` with `Bearer`, re-author `default_pack`'s three rules, add `anchor: Option<PartId>` |
| `mesocosm-core/src/effect_pack/tests.rs` | **Rework** | Curve (`:158` saturation) and refusal tests survive verbatim; the three pole tests rewritten on bearers |
| `mesocosm-core/src/embodiment.rs` | **New** | §2.3 |
| `mesocosm-runtime/src/glyphs.rs` — `owns_effect`, `effect_bases` | **Keep** | Journey-side, bearer-agnostic (`:125-155`) |
| same — `acquired_by`, `AcceptedKind::Uptake`, `absorb_uptake` | **Rework, not discard** | `AcceptedKind` stops being a rule axis and stays evidence. `absorb_uptake` (`:202-234`) is the tree's only flow-reading grant path and the seed of condition rule 2; ruling 5's "a plant earns the glyph by making its living" survives as a *used* condition |
| `mesocosm-runtime/src/glyphs/tests.rs` | **Keep the invariants** | The two read-only tests are axis-independent |
| `bench/trial/journey.rs` | **Rework** | `default_rules`, `Binding`, `Reading`, `glyph()` survive in shape; `pole()`/`pole_label()` (`:75-90`) retire with `Acquiring`; `resolve` asks the embodied reading first |
| `bench/trial.rs` | **Keep** | `marks_withheld`, `refresh_marks`, the budget counter and every probe field survive; two new fields join them |
| `bench/trial/carving.rs`, `uptake.rs` | **Keep** | Placement and pulse geometry are unaffected by the axis change |
| `mesocosm-core` → `wing-glyphs` edge (ruling D1) | **Keep** | Now carries two shared tables instead of one |
| `testing/glyphs/journey.json` | **Keep** | Regenerated at step 0 (`df397e7c183eec55` / `2dc9e1470df7c7c4`); unaffected |

Nothing is discarded outright. The one genuinely retired idea is the
**acquiring-act axis itself**, and the reason is Mark's ruling, not a defect.

---

## 6. Move order for the first slice: express every glyph with a trait

Smallest first; each step green and revertible on its own.

1. **Expression table, shared.** New `wing-glyphs/src/expression.rs` +
   `expression_tests.rs`, exported from `lib.rs`. Pure data: `traits_of`,
   `glyphs_of`, `covers(&Canon)`, JSON round-trip, revision agreement, bounded
   identifiers reusing `identifier()` and `bounded_text()`. *Gate:* the
   wing-glyphs suite, **24 tests today** (canon 6, journey 8, divine 5, pack 5).
2. **Embodied reading, core.** New `mesocosm-core/src/embodiment.rs` +
   `embodiment/tests.rs`: `embodied(&BodyPhenotype, &Registry, &ExpressionTable)`,
   `&`-only, no `World`, no renderer type. *Gate:* core's suite plus
   `tests/move_pins.rs` — the six byte pins, including the state-hash pin
   (`:87`) and the profile schema/version pin (`:96`). No `World` is touched, so
   no pin may move.
3. **The body events, as tests.** In `embodiment/tests.rs`: founding anatomy
   embodies its seeded glyphs; `Grew` and `Grafted` are monotone on the
   recipient; `Severed` removes the tombstoned part's contribution while
   `mosaic()` still answers; a dead body bears nothing. *Gate:* as step 2, plus
   `tests/embodied.rs`, which already pins "what a critter can do is read off
   what it is" (`embodied.rs:13`).
4. **Pack table re-keyed.** `Acquiring` → `Bearer`; `MarkRequest.anchor`;
   `default_pack` re-authored. Curve, refusals and `validate_against` unchanged.
   *Gate:* core's suite; the six move pins.
5. **Runtime query.** `GlyphReading` keeps `owns_effect`; `acquired_by` is
   demoted to evidence and no longer chooses a form. `Trial::glyphs()` stays the
   only door. *Gate:* the **49 runtime tests** (runtime 14, glyphs 10, trial 7,
   clock 6, readings 5, tactile 4, carve 3).
6. **Bench binds expression.** `bench/trial/journey.rs` holds the expression
   table beside the pack and asks the embodied reading first. New probe fields
   `trial-embodied` and `trial-expressing-parts`; `trial-acquired-by` becomes
   `trial-borne-by`. Marks placed as today. *Gate:* `world-trial`,
   `world-trial-play`, `world-trial-reopen`, `uptake-world`. **Pixels do not
   move.**
7. **Marks emit from part faces.** `SurfaceInscription` takes centre and axes
   from the bearing part's `GlyphAnchor` (`shared/isometer/src/anchors.rs:35`).
   **Pixels move here.** *Gate:* the four trial scenarios, plus
   `spatial-coverage`'s **32 viewports byte-identical** — a separate path that
   must not be disturbed. Assert `trial-mark-budget-dropped == 0` against
   `MAX_GLYPH_ANCHORS = 32` (`anchors.rs:16`) and `MAX_SPATIAL_GLYPHS = 128`.
8. **Experienced-but-not-embodied.** The second visual state, and the capture
   showing a severing change what the body shows while the journey is unchanged.
   *Gate:* a new `expression-sever.scenario`.

Workspace gates throughout: `cargo check --workspace --all-features
--all-targets` on the Mesocosm workspace, `cargo fmt` clean, and
`structure-cli` and `population` **named as excluded** with their HEAD failure
hashes (`50e8f3a3a6b6d22d` expected / `8d1d3676ecf24452` got), as the superseded
plan's R3 requires.

---

## 7. Risks and open decisions for Mark

**D1. Is the expression table seeded and shuffled per world, or authored?**
`Canon::shuffled(seed, revision)` (`canon.rs:173`) already permutes the
glyph→effect correspondence per world while preserving the bijection. The
expression table could follow it — every world discovers which traits express
which glyphs — or it could be authored once and stay fixed across worlds.
Consequence: a shuffled table makes the *same* body embody different glyphs in
different worlds, which is a strong discovery mechanic and makes cross-world
`fili` comparisons meaningless. An authored table makes trait-to-glyph part of
the game's fixed vocabulary. **Not taken unilaterally.** This plan assumes
authored, because §7.4's migration rule ("canon changes must not quietly make an
old collection complete under different meanings") is much harder to honour on a
shuffled table that is also a mantling gate.

**R2. The arithmetic does not currently work, and this is the biggest finding.**
The default canon §7.4 wants covers "letters, numbers, and symbols" — call it 62
bases at the low end. The traits that exist are **five** native process
definitions (`process.rs:64-90`), of which only four are ever seeded by geometry
and only one (`Secrete`) requires a development. The bench's preset canon has
**one** glyph (`bench/trial/journey.rs:27`). So `covers(&Canon)` over any real
default canon would need one trait to express a dozen glyphs, which makes
embodiment nearly useless as a mantling gate. Three ways out, all Mark's:
raise the definition count through admitted packs (`mesocosm-phenotype`'s pack
door already admits authored `ProcessDef`s); key expression on a **composed**
trait — the traits brief's rarity ladder, where a trait is a combination of
effects (`traits_and_perception_brief.md`, Mark's message) — so the
combinatorics supply the count; or start Mesocosm's canon deliberately small,
which §7.4 explicitly permits ("a world may use only `a`, `b`, and `c`").

**R3. What can a critter with no expressing trait ever do?** Under §2.3 a body
that expresses nothing in the table embodies nothing, mantles nothing, and — if
experience conditions all require a bearing trait — can never experience
anything either. That is a dead end a player can reach by being cut badly. Three
candidate answers: experience is permanent once had, so only *mantling* is lost
(this plan's assumption, and the one consistent with "you cannot sacrifice a
carving"); a body always bears a floor trait through its root part, which
`Role::Mass → Intake` effectively already guarantees at founding; or an
item-held (`Bearer::Held`) route exists for borgs, which is Mark's ruled tier
and is out of this slice. **Needs a ruling before step 8.**

**R4. Two readings of "what a part does" can disagree.**
`BodyDocument::processes` answers from geometry (`process.rs:452-457`);
`BodyPhenotype::allocations` answers from expressed sites. A development that
moves tissue makes them disagree by design. If embodiment ever reads the
geometric one, a glyph would appear on a part that does not express it.
Mitigation: `embodied` takes `&BodyPhenotype`, never `&BodyDocument`, and a test
asserts the two disagree after an `Expressed` event and that embodiment follows
the phenotype.

**R5. The hagiograph lane is blocked upstream.** `Runtime::end_epoch` has one
caller and it is a unit test, so `WorldRecord` is permanently empty in a real
run (traits brief §2). Condition rule 3's promoted-event source cannot be built
on it until the epoch boundary is wired. Flagged, not taken; rules 1 and 2 need
nothing from it.

**R6. The revision pin becomes load-bearing twice.** G5's rule — live effect
follows the current revision, completion follows the founding one
(`journey.rs:231-236`, `:249-260`) — now applies to the expression table as well
as the pack. A journey founded under revision 1 whose world publishes revision 2
must judge completion against revision 1's canon and embodiment against revision
2's table. Write both lookups revision-aware and pin the reading in a test
before a second revision exists, exactly as the superseded plan's R7 asked.

**R7. Mark's ruled first slice is §2 only.** §3 is design. Building the
condition vocabulary before Mark rules it would be the same mistake the effect
pack plan made in choosing an axis unilaterally.

---

## 8. Done conditions

**Tests.**

1. wing-glyphs `expression`: JSON round-trip; `covers` accepts a canon whose
   every base has a trait and refuses one that does not; a table whose
   `canon_revision` disagrees with the canon's refuses; an entry with an empty
   trait list refuses at `new`; `glyphs_of` returns the many-to-many inverse;
   bounded-text and identifier rules match `canon.rs`'s.
2. `mesocosm-core` `embodiment`: a seeded founding producer embodies exactly the
   glyphs its `Intake` and `Fix` sites express; `Grew` and `Grafted` are monotone
   on the recipient; `Severed` removes that part's contribution while
   `BodyPhenotype::mosaic` still answers for it; a body with every part severed
   embodies the empty set; the reading is `&`-only and byte-identical when run
   twice over one unchanged phenotype.
3. `mesocosm-core` `effect_pack`: the amount curve is monotone, saturating and
   exact at its endpoints (unchanged from the superseded plan); `Bearer::Trait`
   and `Bearer::Journeyed` return their authored forms; `validate` refuses two
   rules on one `(effect, bearer)` pair; `resolve` with `owned: false` refuses.
4. The two read-only invariants survive verbatim: repeated resolution leaves the
   journey byte-identical, and a core-rejected carve yields no grant and no mark.
5. The 24 wing-glyphs tests, the 49 runtime tests and the six move pins stay
   green at every step. §7.4's "16 tests" receipt is corrected to 24 in the same
   session as step 1 — it was already stale at 19.

**Scenarios.**

6. `world-trial`, `world-trial-play`, `world-trial-reopen` and `uptake-world`
   pass at every step; steps 1 to 6 leave their pixels unchanged.
7. `spatial-coverage` keeps all 32 viewports byte-identical to the 2026-09-13
   `acceptance-*` receipt arm through the whole lane.
8. A new `expression-sever.scenario`: bind the journey, let the body experience
   the glyph, sever the expressing part, and show the mark leaving the part face
   while `trial-grants` and the journey snapshot are unchanged.
9. `structure-cli` and `population` named as excluded, with their HEAD failure
   hashes, in every step's green claim.

**Receipts**, under `testing/bench/receipts/2026-09-15/expression/`: for one
run, the expression table, the embodied glyph set per tick, the expressing part
ids, the journey snapshot and `trial-mark-budget-dropped == 0`; a capture pair
before and after a severing, same journey, different embodied set; and a capture
of a body embodying a glyph it has not experienced, drawing nothing, with the
withheld count naming it.

---

## Ruling on experience conditions (Mark, 2026-09-15)

Experience conditions are general predicates over the log, not per-glyph
mechanics. The form is a conjunction: **an encounter with the glyph** (it
was found, seen inscribed, carried, or embodied) **and a deed recorded
among the significant events** that matches the glyph's meaning. "If you
have killed something with fire and you found this glyph, you have
experienced it," rather than "set something marked with this glyph on
fire." The trigger is recorded with **the manner that satisfied it**:
discovery, craft, embodiment or deed, and that manner is what shapes the
divinity later. Conditions are data and what counts may change by canon
revision. The three trigger kinds proposed in section 3 (borne, used,
befallen) are therefore special cases of one predicate vocabulary and do
not stand on their own; the manner set maps onto the kernel's provenance
kinds, which already exist and are what the motif reads. Expression
through traits stays the first Mesocosm slice; experience follows it.

## Rulings on the three findings (Mark, 2026-09-15)

1. **Many more traits.** The five native process definitions are not the
   trait set; the catalogue grows, through admitted definitions and the
   traits brief's composed and rarer traits, until every canon glyph has
   expressing traits. Sizing the catalogue is its own lane in the
   phenotype and traits material, and this plan consumes it.
2. **Authored expression with a pinned shuffle.** The canon's expression
   set is authored; a world may shuffle it by seed, and the shuffled
   mapping is pinned into the world's definition exactly as the canon
   correspondence already is (`Canon::shuffled(seed, revision)` then
   saved, never rerun on an inhabited world), so a shared world keeps one
   consistent set. Open decision D1 is closed this way.
3. **A lineage acquires traits over epochs.** A critter cut down to no
   expressing trait is a dead end only within its epoch: play runs as a
   round in the terrarium and then review and revision on the trait board
   in initiative order by biomass, so the lineage takes traits across
   epochs, and discovery and deed manners need no bearing trait at all.
   Finding 3 is answered by the game's own loop rather than by a rule.

## Rulings on the bench's four choices (Mark, 2026-09-15)

1. **An embodied glyph draws before it is experienced.** Expression is
   visible on the body from birth; experience is the journey's separate
   record and gates ascension, not display. Section 4's sentence that an
   embodied-but-unexperienced glyph draws nothing is withdrawn.
2. **The placeholder expression table stands** (reshape from intake and
   fix, reach from contract and sense, guard from secrete) until the
   catalogue and the authored set replace it.
3. **One bearing part per mark, chosen by record ordinal across the
   bearing parts**, so marks spread over what expresses the glyph.
4. **An anchored mark keeps the anchor's centre and axes and takes only
   the bench's existing display raise**, cleared a hair along the normal
   and lifted in world up, since a mark flush to its face is occluded by
   its own body.

## Findings (2026-09-15)

Measured in the tree today; the rest are cited inline above.

- **There is no `Trait` type.** The closest rule-bearing, part-borne datum is
  `phenotype::mosaic::Site` (`mosaic.rs:68-77`). §1 places the six near misses.
- **`ProvenanceKind::Trait` has existed since G1 and has never been written**
  (`journey.rs:12-21`; every Mesocosm grant is `Event`, `runtime/glyphs.rs:251`).
- **Test counts:** wing-glyphs **24** `#[test]` across four test files,
  mesocosm-runtime **49**. §7.4's receipt says 16 and 44; the superseded plan's
  R2 caught the first drift at 19.
- **Five native process definitions** (`process.rs:64-90`) against a §7.4
  default canon of letters, numbers and symbols. That is R2, and it is the
  slice's real constraint.

## Progress

- **2026-09-15.** Plan written. No code moved, nothing committed.
- **2026-09-15, steps 1 to 7 done, committed together once the tree was
  green.** wing-glyphs: `expression.rs` (238), a per-canon-revision table
  with `traits_of`, `glyphs_of`, `covers`, JSON with limits, and
  `shuffled(seed, revision)` pinned like the canon's, with a literal
  receipt; 24 to 28 tests. mesocosm-core: `embodiment.rs` (111) with
  `embodied`, `expressed_traits`, `bearing_parts` over `BodyPhenotype` and
  the process registry, tested for founding, growth, grafting, severing
  and death; `effect_pack.rs` re-keyed to `Bearer { Embodied, Journeyed,
  Held }` with `MarkRequest.anchor` (the trait identity travels on the
  anchor, since a rule keyed on a trait id cannot be validated as a
  finite set), Held declared and unauthored; the six pins hold.
  mesocosm-runtime: `embodied_glyphs(&World, &table)` through the trial's
  one door; `acquired_by` and the uptake kind are evidence only; 50 tests;
  the bound producer embodies from tick zero while its journey is empty.
  Bench: `trial/journey.rs` holds a placeholder three-glyph table over the
  five native process ids, asks the body first and the journey second;
  probe fields `trial-embodied`, `trial-expressing-parts`,
  `trial-borne-by`, `trial-anchor-fallbacks`; marks re-seat on the
  bearing part's glyph anchor, cleared along the normal and raised by the
  bench's marker height, spread across bearing parts by record ordinal.
  Gates: world-trial, world-trial-play and uptake-world pass with no
  assertion weakened; spatial-coverage's 32 captures byte-identical to the
  2026-09-14 set; acceptance passes; budget drops zero; one anchor
  fallback in uptake-world where a body expresses on 33 parts against the
  32-anchor cap. The dead bound body in world-trial's late captures
  embodies nothing and every mark withdraws, the gate showing as designed.
  Four choices the agent made await Mark: an embodied glyph draws before
  it is experienced (the plan's §4 said otherwise); the placeholder glyph
  ids and grouping; one bearing part per mark chosen by ordinal; the
  world-up display raise on an anchored mark.
