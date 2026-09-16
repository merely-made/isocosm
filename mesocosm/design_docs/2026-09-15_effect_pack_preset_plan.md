# Effect pack preset plan

**Status: plan, 2026-09-15.** Assessment only; no code moved, nothing
committed. Implements gate **G3** of the
[general model plan §7.4](2026-08-06_general_model_plan.md#74-glyph-canon-the-journey-and-divinity-2026-09-13)
("admit a default effect pack and two contrasting journey rules through
product adjudication") as one replayable specimen-bench preset, over the
bench and trial the
[presentation plan](2026-09-11_orthographic_voxel_presentation_plan.md#bench-c-visible-disposable-world-trial)
already owns. Ruled by Mark, 2026-09-15.

## 0. The slice

One complete, replayable specimen-bench preset in which the live trial
markers stop being hardcoded and instead read the bound journey's grants
through a default effect pack.

1. Marker glyph, colour and form come from the bound journey's grants
   through a default effect pack owned by `mesocosm-core`, beside the
   existing `effect_experiment`. **An event whose glyph is not yet acquired
   paints nothing** — the first visible consequence of the journey.
2. Amount sensitivity from the accepted record: uptake milligrams set the
   pulse's size or lifetime, so two recipients differ for a stated reason.
3. Two contrasting journey rules on one glyph, authored, cited to the
   acquisition evidence, visibly different in form.
4. **Invariant, one direction only.** Markers read grants; grants come only
   from accepted history; nothing drawn can award anything.

**Explicit non-scope.** Durable `World` changes; playable reincarnation;
wish execution; divine spending; the T3 viewport embedding; and the
wing-glyphs canon's `display` text driving stroke shapes — that is a later
feature, and `isometer::Stroke` (three variants,
`shared/isometer/src/glyphs.rs:16-21`) is the renderer's whole vocabulary
today.

---

## 1. The current wiring

Two flows run past each other. The bench draws from `Trial`'s per-step
projections; the journey is fed from the same `Trial`'s history by an
adapter the bench never calls.

| Accepted event | Today's marker (hardcoded) | Grant path (runtime adapter) | Where they fail to meet |
| --- | --- | --- | --- |
| `Event::Moved {from,to}` (`mesocosm-core/src/history.rs:91-95`) | `Glyph::Slashes`, colour `[0.45,0.95,0.8,1.]`, camera-facing, interpolated `from`→`to`, 8-tick life (`bench/trial.rs:119-137`) | `AcceptedKind::Moved` → `EventGrant.glyph` → `Journey::grant` (`mesocosm-runtime/src/glyphs.rs:175`, `:143`) | Bench never reads the journey |
| `Event::Fed {mass_mg}` (`history.rs:84-89`) | `Glyph::Quotes`, colour `[1.,0.75,0.3,1.]`, same block (`bench/trial.rs:121-136`) | `AcceptedKind::Fed`, `mass_mg > 0` (`glyphs.rs:176`) | Same; `mass_mg` never reaches the mark |
| `Event::Carved {at,removed}` (`history.rs:123-127`) | `Stroke::Slashes`, colour `[1.,0.45,0.2,1.]`, camera-facing at the recorded centre (`bench/trial/carving.rs:104-106`) | `AcceptedKind::Carved`, `removed > 0` (`glyphs.rs:172-174`) | Same; `removed` never reaches the mark |
| Soil uptake `RecordedFlow` (`flow.rs:231`, `Process::Uptake`, `Account::Soil`) | `Stroke::Backticks`, colour `[0.6,0.85,0.3,1.]`, world-plane, run-anchored rise (`bench/trial/uptake.rs:81-98`) | **None.** `GlyphReading::absorb` iterates `history.log()` only (`glyphs.rs:127-128`) | Uptake is a flow, not an `Event`; it has no grant path at all |

The precise breaks, with evidence:

- **The bench holds no journey.** `enable_glyphs`, `Trial::glyphs()` and
  `wing_glyphs` appear zero times anywhere in `mesocosm-genet`. The only
  callers are `mesocosm-runtime/src/runtime/trial.rs:117,126` and
  `crates/mesocosm-runtime/examples/glyph_journey.rs:28,88`.
- **Colour and stroke are literals** at `bench/trial.rs:132-136`,
  `bench/trial/carving.rs:104-106`, `bench/trial/uptake.rs:91-96`. Form is
  a literal too: `GlyphOrientation::CameraFacing` for movement, feeding and
  carving; `WorldPlane` for uptake.
- **The kernel records no amount.** `GrantRecord`
  (`shared/wing-glyphs/src/journey.rs:66-74`) is glyph, provenance, tick,
  life, policy, canon revision. There is no milligram and there should not
  be: amount stays product-side, keyed by `(tick, sequence)`.
- **The grant record does carry the event.** `GlyphEvidence.event: Event`
  (`glyphs.rs:57`) means `Fed.mass_mg` and `Carved.removed` are already in
  the reading. Uptake milligrams are not, and cannot be without a second
  absorb path.
- **Effect ids are inert.** `Canon::effect` (`canon.rs:164-169`) returns a
  string. Nothing in the tree consumes it; `effect_reference_executed:
  false` is printed as a standing claim
  (`examples/glyph_journey.rs:105`).
- **The two mark vocabularies are already joined**, one way:
  `section.rs:49-55` maps `mesocosm_core::effect_experiment::Glyph` to
  `isometer::Stroke`. That mapping is the seam the pack should target, so
  `mesocosm-core` never names a renderer type.

---

## 2. The effect pack

### 2.1 Where the declaration lives

**`shared/wing-glyphs`, a new `pack` module beside `canon`.** Reasons:
§7.4 already assigns wing-glyphs "canon validation … no graphics or
product dependencies"; `EffectId` is already a wing-glyphs type
(`canon.rs:15`) and a declaration keyed by `EffectId` living in another
crate splits one vocabulary across two homes; wing-glyphs depends only on
`serde` and `serde_json` (`shared/wing-glyphs/Cargo.toml`), so nothing
product-shaped arrives with it; and a fourth shared crate for ~130 lines of
plain data is what DOC_POLICY §1 and the consolidation rule refuse. Share
the **declaration** now, keep **execution** in Mesocosm until Paredros or
Isometry consumes one, then extract.

**Decision for Mark (D1).** `mesocosm-core` does *not* depend on
wing-glyphs today — `mesocosm-runtime/Cargo.toml` says so in as many words
("core knows nothing of either crate"). An execution table in
`mesocosm-core` that names `EffectDeclaration` adds that edge. Two shapes:

- **(a) Core sees the declaration.** Add `wing-glyphs` to
  `mesocosm-core`. One new edge, pure data, no duplication.
- **(b) Runtime owns the join.** Core's table is keyed by plain effect-id
  `&str` and restates behaviour/receiver/cost locally; `mesocosm-runtime`,
  which already depends on both, checks declaration against table. The
  one-way rule survives; the vocabulary is stated twice.

Recommended **(a)**, because (b)'s duplicated vocabulary is exactly the
drift DOC_POLICY §2 exists against. Not taken unilaterally.

### 2.2 Declaration types (shared, no execution)

```rust
// shared/wing-glyphs/src/pack.rs — illustrative, not compile-ready.
// Data only. Nothing here runs an effect, draws, or grants.

/// What form an effect asks a product to give it.
pub enum BehaviourKind { Inscribe, Emit, Trail, Enclose }
/// What class of thing it acts on. Products map these to their own nouns;
/// the kernel knows no terrain, body or organism.
pub enum ReceiverClass { Terrain, Body, Bearer, None }
/// A named quantity a product must be able to meter. Kernel-opaque.
pub struct CostUnit { pub id: String, pub label: String }
pub enum CostShape { Free, PerUse { unit: CostUnit }, WhileSustained { unit: CostUnit } }

pub struct EffectDeclaration {
    pub effect: EffectId,
    /// Opaque display text, as on GlyphDefinition. Not a stroke shape.
    pub display: String,
    pub behaviour: BehaviourKind,
    pub receiver: ReceiverClass,
    pub cost: CostShape,
    /// Named holes a product fills: "{actor} inscribes {amount} {unit}".
    /// The kernel neither formats nor executes it.
    pub explanation: String,
}
pub struct EffectPackSpec {
    pub version: u32, pub id: String, pub revision: u64,
    pub declarations: Vec<EffectDeclaration>,
}
/// Arc<EffectPackSpec> plus a BTreeMap index, exactly as Canon is built.
pub struct EffectPack { /* .. */ }

impl EffectPack {
    pub fn new(spec: EffectPackSpec) -> Result<Self, String>;
    pub fn declaration(&self, effect: &str) -> Option<&EffectDeclaration>;
    /// Every base glyph's effect in `canon` has a declaration here.
    /// Pure validation; it executes nothing.
    pub fn covers(&self, canon: &Canon) -> Result<(), String>;
    pub fn to_json(&self) -> Result<String, String>;
    pub fn from_json(json: &str) -> Result<Self, String>;
}
```

### 2.3 Execution table (Mesocosm)

```rust
// crates/mesocosm-core/src/effect_pack.rs — beside effect_experiment.
// Illustrative. Emits a request; it does not draw and does not touch World.
use crate::effect_experiment::Glyph;

/// The journey-rule axis (§3), and the forms its poles return as.
pub enum Acquiring { Carved, Fed, Moved }
pub enum MarkForm { SurfaceInscription, SustainedEmission, PathTrail }

/// One authored rule: this effect, acquired this way, returns like this.
pub struct PackRule {
    pub effect: String,
    pub acquired_by: Acquiring,
    pub form: MarkForm,
    pub stroke: Glyph,
    pub color: [u8; 4],
    pub cost: CostShape,
    /// Free text naming the acquisition evidence the rule answers to.
    pub citation: String,
}

/// The amount the accepted record supplies. Voxels: Event::Carved.removed.
/// MealMass: Event::Fed.mass_mg. UptakeMass: RecordedFlow.amount_mg under
/// Process::Uptake. None: Event::Moved.
pub enum Amount { Voxels(u32), MealMass(u64), UptakeMass(u64), None }

/// Presentation-neutral marking order. No renderer type appears here.
pub struct MarkRequest {
    pub form: MarkForm,
    pub stroke: Glyph,
    pub color: [u8; 4],
    pub at: [i32; 3],
    /// Movement's arrival, for PathTrail. None otherwise.
    pub to: Option<[i32; 3]>,
    /// 1000 = the preset's base size. Derived from `Amount`, never RNG.
    pub scale_permille: u16,
    pub lifetime_ticks: u16,
    pub explanation: String,
}
pub enum Refusal { NotAcquired { effect: String }, NoRule { effect: String } }
pub struct EffectPackTable { rules: Vec<PackRule> }

impl EffectPackTable {
    pub fn default_pack() -> Self;
    pub fn validate(&self) -> Result<(), String>;
    /// `owned` is the journey's answer, passed in. This function cannot
    /// reach a Journey, cannot grant, and cannot mutate anything.
    pub fn resolve(
        &self, effect: &str, owned: bool, acquired_by: Acquiring,
        at: [i32; 3], to: Option<[i32; 3]>, amount: Amount,
    ) -> Result<MarkRequest, Refusal>;
}
```

The bench turns a `MarkRequest` into a `SpatialGlyph` through the existing
`section::stroke()` mapping (`section.rs:49-55`) and the existing form
geometry, so `mesocosm-core` still names no renderer type and
`shared/isometer` gains nothing.

### 2.4 Grant to marker

| Field | Source |
| --- | --- |
| stroke | `PackRule.stroke` → `section::stroke()` → `isometer::Stroke` |
| colour | `PackRule.color`, byte RGBA → the `[f32;4]` opaque form `GlyphLayer::set` requires (`isometer/src/glyphs.rs:171-187`) |
| form | `MarkForm` → `CameraFacing` or `WorldPlane` plus the placement below |
| placement | `SurfaceInscription`: the recorded `Carved.at`, world-plane, with the coplanar offset the spatial extension already uses. `SustainedEmission`: the recipient's recorded position, rising, as `uptake::pulse`. `PathTrail`: the `from`→`to` interpolation at `bench/trial.rs:117` |
| lifetime | `PackRule.cost` shape: `PerUse` → fixed 8 ticks; `WhileSustained` → the run anchor in `uptake::Pulses` (`bench/trial/uptake.rs:41-75`), retired when flow stops; `Free` → fixed 8 ticks |
| size | base `marker_size` × `scale_permille`/1000 × the existing age falloff |

### 2.5 "Paints nothing until acquired", per marker kind

`resolve` returns `Refusal::NotAcquired` and the bench emits no
`SpatialGlyph`. Against the four kinds that exist today:

1. **Movement slashes** (`bench/trial.rs:119-137`) — no mark until the
   movement glyph is acquired. The first accepted `Moved` *is* the
   acquisition, so the first movement is unmarked and every later one is
   marked. That asymmetry is the visible consequence.
2. **Feeding quotes** (same block) — likewise on the first accepted `Fed`.
3. **Carving marks** (`bench/trial/carving.rs:91-110`) — likewise on the
   first `Carved` with `removed > 0`. The `events`/`removed` counters keep
   counting; only the mark is withheld.
4. **Uptake pulses** (`bench/trial/uptake.rs:81-98`) — uptake has **no
   grant path** (§1), so under this preset the pulse binds to the *feeding*
   glyph: a soil uptake by a body that has never fed paints nothing. Stated
   in the preset, not invented in the adapter. The alternative is a second
   absorb path over `RecordedFlow`, which `Trial` does not have and G3 does
   not need (R8).

Probe fields report the facts either way — `trial-uptake-mg`,
`trial-carving-removed`, `trial-activity` are unchanged — so a run can show
many events and few marks, and the receipt proves the withholding.

### 2.6 Amount

Three sources, all from the accepted record, all integer:

- **uptake mg** — `RecordedFlow.record.amount_mg` (`flow.rs:231`), reaching
  the bench as `TrialUptake.record` (`runtime/trial.rs:42`, consumed at
  `bench/trial.rs:83`). Drives `scale_permille` **and** a longer pulse
  lifetime, so two recipients differ for a stated reason.
- **carve voxels** — `TrialCarve.removed: u32` (`runtime/trial.rs:58`).
  Drives `scale_permille` of the inscription.
- **meal mass** — `Event::Fed.mass_mg` (`history.rs:87`), already inside
  `TrialActivity.event` and `GlyphEvidence.event`. Drives `scale_permille`
  of the trail.

Mapping is a fixed integer curve on the core side (a saturating
`1000 + min(amount, cap) * span / cap`), never a float and never an RNG
draw, so replay reproduces it exactly. The journey stores none of it.

---

## 3. The journey rules

### 3.1 The axis: the acquiring act

**Poles: `Carved`, `Fed`, `Moved`.** Three, not two.

| Pole | Evidence the simulation already records |
| --- | --- |
| Carved | `Event::Carved { organism, at, removed }`, `removed > 0` (`history.rs:123-127`; selected at `glyphs.rs:172-174`) |
| Fed | `Event::Fed { eater, from, mass_mg, kind }`, `mass_mg > 0` (`history.rs:84-89`; `glyphs.rs:176`) |
| Moved | `Event::Moved { organism, from, to }`, `from != to` (`history.rs:91-95`; `glyphs.rs:175`) |

The pole is carried in the grant already: `AcceptedKind`
(`glyphs.rs:15-21`) selects it, `GlyphEvidence.event` keeps the whole
event, and `Provenance.context` records `"{kind:?}; organism={}"`
(`glyphs.rs:148`). Nothing new is recorded to read this axis.

**Why this axis, not the provenance-kind axis.** `ProvenanceKind`
(`journey.rs:12-21`) has eight variants, but Mesocosm's adapter writes
`ProvenanceKind::Event` for every grant it ever makes (`glyphs.rs:145`).
That axis is degenerate here: one pole, so `Journey::motif()`
(`journey.rs:381-394`) returns exactly one group for any Mesocosm journey.
Authoring contrasting rules on an axis the product cannot vary would be
authoring against a constant.

**Why not `guise`.** `Organism.guise: Kingdom`
(`organism.rs:175`, `organism/kingdom.rs:71-79`) does give three poles, and
they are recorded. Two objections. It is a claim, not a reading — a mimic's
guise deliberately disagrees with `kingdom()` (`organism.rs:437`,
`:461`) — so a rule citing it cites something the accepted record
contradicts. And it is world state on the body, not a fact in the event, so
a journey rule keyed on it would be re-read at draw time from a `World` the
marker is not allowed to consult. Keep it as a **second axis** for later:
Mark's motif-as-a-vector framing wants several axes, and guise is the
natural second one once a rule may ask about the bearer as well as the act.

### 3.2 The three authored rules, on one glyph

One base glyph, one effect id, three acquiring-act rules. The contrast
Mark named is the first two; the third is the pole the simulation already
has and the bench already draws.

| Rule | Form | Cost or obligation | Citation | Visible difference |
| --- | --- | --- | --- | --- |
| **Acquired by carving** | `SurfaceInscription` — world-plane strokes at the recorded carve centre, offset off the surface | `PerUse { unit: "mesocosm:voxel" }`. Every return spends removed voxels; nothing returns for free | The `Carved` grant's `evidence` string, `mesocosm.trial/{baseline}/{post}/history/{sequence}` (`glyphs.rs:139-141`), plus its `canon_revision` | A stable mark that stays put on the ground and does not follow the body |
| **Acquired by feeding** | `SustainedEmission` — camera-facing strokes rising off the recipient, run-anchored | `WhileSustained { unit: "mesocosm:mass_mg" }`. It lasts only while feeding continues; a one-tick gap retires it (`uptake.rs:41-75`) | The `Fed` grant's evidence and revision | A pulsing emission that dies the moment the flow stops |
| **Acquired by moving** | `PathTrail` — camera-facing strokes interpolated `from`→`to` | `Free`, fixed 8-tick life | The `Moved` grant's evidence and revision | A trail between two recorded places, tied to neither |

Each rule's `explanation` template names its own citation, so the bench can
print why a mark looks the way it does without the kernel formatting
anything. Rules are authored data in the preset file, not inferred from
event names — §7.4's "authored possibilities, not judgments inferred from
event names".

---

## 4. The preset

**One replayable preset is one JSON file** carrying, in one document:

```text
version, id
rules:      GlyphRules — canon spec, individual, bound organism,
            unlock thresholds, EventGrant list (runtime/src/glyphs.rs:33-42)
pack:       EffectPackSpec (declarations) + PackRule list (execution)
seed:       the specimen world seed and generation request identity
intents:    the fixed trial trace — Idle and Carve, in order
marker:     marker_height, marker_size, show_marks, show_uptake
baseline:   the specimen world state_hash
```

**Saved and reopened** by the pattern `bench/spatial/saved.rs` already
proves: a `#[serde(deny_unknown_fields)]` `Saved` struct with a `validate`,
written to `export_directory` via `tempfile` and kept
(`bench/effects.rs:146-168`, `bench/spatial/saved.rs:35-82`), reopened by a
`--preset FILE` flag alongside `--effect-experiment`
(`mesocosm-genet/src/main.rs:98-105`). Reopen refuses unless the specimen
world hash matches, exactly as `reopen_spatial` does
(`bench/spatial/saved.rs:108-115`).

**Replay reproduces identical marks and identical journey records** on the
existing mechanism: `Trial`'s replay source is its immutable baseline world
plus the applied trace, not seed/count
(`mesocosm-runtime/src/runtime/trial.rs:89-102,148-150`), and
`GlyphReading::reset` rebuilds the reading from the same baseline and rules
(`glyphs.rs:122-125`). Three things must then agree across a replay:

1. **Hashes.** `trial-source-hash` and `trial-hash`
   (`bench/trial.rs:210-213`).
2. **The journey.** `Journey::from_snapshot` already re-derives state from
   transitions and refuses a snapshot that disagrees
   (`journey.rs:454-488`). The preset's receipt carries
   `GlyphReading::records()` and the journey snapshot.
3. **The marks.** Every input to a `MarkRequest` is an integer from the
   accepted record plus a fixed table, so the resulting `SpatialGlyph`
   list is a pure function of `(trace, rules, pack, tick)`. The bench
   already gates on this: the spatial-coverage run compares 32 viewports
   byte-for-byte.

---

## 5. Ownership and boundaries

| Owner | Holds |
| --- | --- |
| `shared/wing-glyphs` | The canon, the journey, and the **declaration** (`pack.rs`). No execution, no drawing, no product nouns. |
| `mesocosm-core` | The **execution table** and the default pack, in `effect_pack.rs` beside `effect_experiment.rs`. Emits `MarkRequest`; names no renderer type; touches no `World`. |
| `mesocosm-runtime` | The grant adapter, unchanged in kind: `GlyphReading` maps accepted history to grants (`glyphs.rs`). Gains a read-only query the bench can ask ("is this effect owned?"). |
| `mesocosm-genet` bench | Presentation. Turns `MarkRequest` into `SpatialGlyph` through `section::stroke()` and the existing forms. |
| `shared/isometer` | Unchanged. `Stroke`, `SpatialGlyph`, `glyph_anchors` gain nothing. |

**Nothing durable in `World`.** No new field, no new saved byte. §7.4 is
explicit: "not a new field in durable `World` or a playable reincarnation
feature."

**The read-only direction, as an invariant.** *A drawn marker cannot create
a grant, and a rejected action cannot grant.* Proven by:

- **Existing:** `opted_in_carve_grant_has_exact_core_provenance_and_replays`
  and `movement_and_feeding_rules_consume_actual_actor_events_only`
  (`mesocosm-runtime/src/glyphs/tests.rs:53,157`) — only actual accepted
  actor events grant, and `accepted()` (`glyphs.rs:170-179`) filters
  `removed > 0`, `from != to`, `mass_mg > 0`, so a refused or null action
  produces nothing.
- **New:** a test that resolves a full marker set N times over one
  unchanged reading and asserts `journey().grants().len()` and the journey
  snapshot are byte-identical before and after. `EffectPackTable::resolve`
  takes `owned: bool` by value and holds no `&mut` anything, so the
  compiler carries most of this; the test states it anyway.
- **New:** a test that a `Carve` refused by core (`last_outcomes` shows the
  rejection, `removed == 0`) yields no grant and no mark.

---

## 6. Move order

Each step is green and revertible on its own, smallest first.

1. **Declaration types.** New `shared/wing-glyphs/src/pack.rs` +
   `pack_tests.rs`, exported from `lib.rs`; pure data and validation
   including `covers(&Canon)`. *Gate:* the wing-glyphs suite (19 `#[test]`
   today, against §7.4's claimed 16 — R2).
2. **Execution table.** New `crates/mesocosm-core/src/effect_pack.rs` +
   `effect_pack/tests.rs`: `PackRule`, `Amount`, `MarkRequest`, `resolve`,
   `default_pack`. D1 settles whether it names `EffectDeclaration` or a
   local mirror. *Gate:* core's suite; no `World` touched, so no state-hash
   movement.
3. **Runtime query.** `GlyphReading::owns_effect(&str) -> bool` over
   `Journey::owns_base` + `Canon::effect`; `Trial::glyphs()` stays the only
   door. *Gate:* the 44 runtime tests.
4. **Bench binds the journey.** `WorldTrial::new` calls
   `driver.enable_glyphs(rules)` from the preset and holds the pack table;
   marks still hardcoded. New probe fields `trial-journey`, `trial-grants`,
   `trial-marks-withheld`. *Gate:* `world-trial`, `world-trial-play`,
   `world-trial-reopen`, `uptake-world`. Pixels do not move yet.
5. **Marks read the pack.** `refresh_marks` (`bench/trial.rs:104-171`),
   `carving::marks` and `uptake::pulse` take stroke, colour, form and
   lifetime from `resolve`. **Pixels move here.** *Gate:* the four trial
   scenarios, plus `spatial-coverage`'s 32 viewports staying byte-identical
   — the spatial preview is a separate path and must not be disturbed.
6. **Amount.** `scale_permille` from uptake mg, carve voxels and meal mass;
   uptake lifetime from mg. *Gate:* `uptake-world` (two recipients visibly
   differ), plus a pure test on the integer curve.
7. **Preset file.** `Saved` + `validate` + `--preset FILE`, modelled on
   `bench/spatial/saved.rs`. *Gate:* a new
   `testing/bench/preset-reopen.scenario`: save, exit, reopen, identical
   hashes and identical viewport.
8. **Invariant tests and receipts.** The two tests in §5, the receipt
   directory, and the progress note.

Workspace gates throughout: `cargo check --workspace --all-features
--all-targets` on the Mesocosm workspace, and `cargo fmt` clean.

---

## 7. Risks

**R1. The `journey.json` baseline is stale — verified today.**
`mesocosm/testing/glyphs/journey.json` records
`baseline_hash 765c055b377dfb62` / `final_world_hash ea76e63d1fb9fa13`.
Running `cargo run -p mesocosm-runtime --example glyph_journey` on 2026-09-15
produces `df397e7c183eec55` / `2dc9e1470df7c7c4`. Everything else in the
receipt is unchanged: same actor, same `carve_target [0,12,0]`, same
`removed: 15`, same `demo:earth` acquisition, same measurement total. So
worldgen moved under the fixture and the glyph lane did not. The fixture
must be regenerated in step 1 or 2, before any preset receipt cites a hash,
or the preset will bake a second stale number.

**R2. Test-count drift in the §7.4 receipt.** The plan's first
implementation receipt says wing-glyphs passes **16 tests**; the tree now
carries **19** `#[test]` attributes across `canon_tests.rs`,
`journey_tests.rs` and `divine_tests.rs`. The count moved when the
`canon_revision` / `grant_at_revision` / `correspondence_diff` work landed
for G5. Update the receipt in the same session as step 1, or the gate
number in this plan is checked against a stale claim.

**R3. Known scenario drift, already recorded, not this lane's.**
`structure-cli.scenario` fails at HEAD on a generated-start hash
(`50e8f3a3a6b6d22d` expected, `8d1d3676ecf24452` got); `population.scenario`
fails and flakes identically at HEAD. Both are in the
[isometer extraction plan](2026-09-14_isometer_extraction_plan.md) steps 5
and 7 and the [family plan](2026-09-14_isometer_family_plan.md) risk 1.
Neither gates this lane, but both must be **named as excluded** in every
green claim rather than passed over — R1 shows worldgen has in fact moved,
so one of these may stop being upstream fixture drift.

**R4. The 128-glyph and 32-anchor ceilings.** `MAX_SPATIAL_GLYPHS = 128`
(`isometer/src/glyphs.rs:12`) and `MAX_GLYPH_ANCHORS = 32`
(`isometer/src/anchors.rs:16`). The trial already drops the oldest marks
over budget and reports it (`trial-mark-budget-dropped`,
`bench/trial.rs:163-170`). New pressure from both directions: a surface
inscription wants an anchored face per carving site, and withholding
unacquired marks changes *which* marks survive the sort. The preset caps
its own population, and the receipt must assert
`trial-mark-budget-dropped == 0` — otherwise a replay could be
byte-identical for the wrong reason.

**R5. The saved-request format is a settings receipt, not a world save.**
`reopen_spatial` refuses unless world hash, pose, camera, isolation and
complete part identity match (`bench/spatial/saved.rs:108-115`). A preset
carrying rules, pack, seed and intents is closer to a world save than
anything the bench has saved. Keep the same discipline —
`deny_unknown_fields`, an explicit `validate`, a loud hash check — and do
not let it become a durable save by accident; G4 owns that.

**R6. Determinism of amount-driven animation under replay.** The marks
already mix two clocks: simulation ticks drive age and the run anchor, but
`WorldTrial::advance` steps from `Instant::elapsed()`
(`bench/trial.rs:176-177`) and `Spatial::advance` from wall time
(`bench/spatial.rs:107-111`). Amount-driven size and lifetime must depend
only on the tick and the integer record, never on frame count. The preset
therefore replays through **Step world**, not **Play world**, and the
scenario must say so.

**R7. Canon revision stamps (G5) the preset must honour.** (Relabeled
2026-09-15: hagioglyph now names the whole divinity organ, §7.4; the
time-varying glyph is G5's canon revision.) G5 is already in
the kernel: `Acquisition`/`GrantRecord.canon_revision` (`journey.rs:61,73`),
`grant_at_revision` refusing a revision older than founding
(`journey.rs:258-260`), `from_snapshot` back-filling `0` to founding
(`journey.rs:427-440`), and `Canon::correspondence_diff`
(`canon.rs:203-224`). §7.4's rule: completion and the ascension basis are
judged against the **founding** revision, while the live effect of an owned
glyph follows the **current** one. So `PackRule` lookup must key on the
current canon's effect for the glyph, not the effect recorded at
acquisition — otherwise one glyph acquired under two revisions would
wrongly draw the same mark. The preset pins a single revision (the adapter
calls `Journey::grant`, `glyphs.rs:143`), so this is latent; write the
table revision-aware and pin the reading in a test before a second revision
exists.

**R8. Uptake has no grant path.** §2.5 resolves it by binding the pulse to
the feeding glyph. Granting on uptake in its own right means a second
absorb path over `RecordedFlow` and a fourth `AcceptedKind` — a change to
the kernel-facing adapter, outside this slice. Flagged, not taken.

---

## 8. Done conditions

**Tests.**

1. wing-glyphs `pack`: a declaration round-trips JSON; `covers` accepts a
   canon whose every base effect is declared and refuses one missing a
   declaration; an empty pack over a non-empty canon refuses; bounded-text
   and identifier rules match `canon.rs`'s.
2. `mesocosm-core` `effect_pack`: `resolve` with `owned: false` returns
   `Refusal::NotAcquired` for all three poles; each pole returns its
   authored form, stroke and colour; the amount curve is monotone,
   saturating and exact at its endpoints; `validate` refuses two rules on
   one `(effect, acquired_by)` pair.
3. Runtime: `owns_effect` agrees with `Journey::owns_base` through
   `Canon::effect`, including a variant id resolving to its base.
4. The two invariant tests in §5: repeated resolution leaves the journey
   byte-identical; a core-rejected carve yields no grant and no mark.
5. The 44 runtime tests and the wing-glyphs suite stay green, with §7.4's
   kernel count corrected to the real number.

**Scenarios.**

6. `world-trial`, `world-trial-play`, `world-trial-reopen` and
   `uptake-world` pass at every step; steps 1-4 leave their pixels
   unchanged.
7. `spatial-coverage` keeps all 32 viewports byte-identical to the
   2026-09-13 `acceptance-*` receipt arm through the whole lane.
8. A new `preset-reopen.scenario`: save, exit the trial, reopen with
   `--preset FILE`, and reproduce `trial-source-hash`, `trial-hash`, the
   grant list and the viewport exactly.
9. `structure-cli` and `population` named as excluded, with their HEAD
   failure hashes, in every step's green claim.

**Receipts**, under `testing/bench/receipts/2026-09-15/effect-pack/`.

10. For one preset run: accepted events per kind, the grant list, the
    withheld-mark count, `trial-mark-budget-dropped == 0`, and the
    per-marker `MarkRequest` list.
11. A capture of one specimen where the first accepted event of each kind
    is unmarked and the second is marked — the journey made visible.
12. A capture of two recipients whose different uptake milligrams carry
    visibly different pulses, with both figures in the receipt.
13. `testing/glyphs/journey.json` regenerated, its new hashes recorded
    here, and the stale pair kept in R1 as the evidence of the drift.

---

## Rulings (Mark, 2026-09-15)

1. **D1: mesocosm-core depends on wing-glyphs.** One new pure-data edge;
   the execution table in core names the shared declaration once and
   wing-glyphs stays product-free.
2. **The axis is the acquiring act with three poles**, carved, fed and
   moved, all already selected by the runtime adapter's event filter:
   carving-earned returns as a per-use surface inscription, feeding-earned
   as an emission that lives only while feeding continues, movement-earned
   as a free trail.
3. **Uptake binds to the feeding glyph.** No fourth accepted kind; uptake
   milligrams drive the feeding-earned emission's size and lifetime.
5. **Uptake is a producer's feeding (2026-09-15, from the step 5 finding
   that the bound organism in every trial scenario is a producer that
   never moves, feeds or carves).** The runtime adapter gains an uptake
   kind that reads recorded flows and grants through the feeding pole; a
   plant earns the glyph by making its living.
6. **Form follows the acquiring act.** Ownership is journey-wide, and every
   mark of the effect takes the rule of the pole the journey acquired it
   through: acquired by carving, it returns as an inscription on every
   use; by feeding, as an emission; by movement, as a trail. One glyph,
   different divinities. The plan's per-kind gating in §2.5 is dropped.
7. **The mass cap is 32 mg**, calibrated to the recorded 3 to 28 mg range
   of meals and uptake, in place of the plan's 4,000.
4. **The glyph journey receipt is regenerated as step 0** before any preset
   receipt cites a hash, with the worldgen change that moved it noted.

**Superseded, 2026-09-15 (same day).** Mark ruled later that day that
experiencing the glyph is the main thing, that a glyph is *had* rather than
performed, and that a critter's access is **embodied** — a trait or part bears
it. That replaces the acquiring act with the **bearer** as the journey-rule
axis, so **§3 in full and rulings 2, 5 and 6 above are superseded** by the
[glyph expression plan](2026-09-15_glyph_expression_plan.md). The rest of this
plan — §§0-2 and 4-8, and rulings 1, 3, 4 and 7 — remains the record of steps 0
to 3, which landed. The new plan says file by file what is kept, reworked or
retired; nothing landed is discarded, and `Acquiring` becomes `Bearer`.

## Findings (2026-09-15)

Verified in the tree today; the rest are cited inline above.

- **The bench has never touched the journey.** `enable_glyphs`,
  `Trial::glyphs()` and `wing_glyphs` do not appear in `mesocosm-genet` at
  all. G2's consumer is the runtime and the example, not the bench.
- **`journey.json` is stale** (R1), and **wing-glyphs carries 19 tests
  against §7.4's claimed 16** (R2). Both measured, not inferred.
- **`mesocosm-core` does not depend on wing-glyphs**;
  `mesocosm-runtime/Cargo.toml` states the one-way rule explicitly (D1).
- **The provenance-kind axis is degenerate in Mesocosm**: every grant the
  adapter makes is `ProvenanceKind::Event` (`glyphs.rs:145`), so
  `Journey::motif()` returns one group for any Mesocosm journey.
- **`GrantRecord` carries no amount and should not.** Amount stays
  product-side keyed by `(tick, sequence)`.

## Progress

- **2026-09-15.** Plan written. No code moved, nothing committed. The one
  runtime action taken was running the existing
  `mesocosm-runtime` example `glyph_journey` to test the `journey.json`
  fixture, which produced the R1 measurement above.
- **2026-09-15, step 0 done (a4d28d4).** `testing/glyphs/journey.json`
  regenerated: baseline `df397e7c183eec55`, final `2dc9e1470df7c7c4`,
  every other field unchanged; worldgen commits of 2026-09-13 moved the
  seed-0 world, not the glyph lane.
- **2026-09-15, steps 1 to 3 done.** wing-glyphs gains `pack.rs` (204
  lines): `EffectDeclaration`, `BehaviourKind`, `ReceiverClass`,
  `CostShape` with `CostUnit`, `EffectPackSpec` with a defaulted
  `PackLimits`, and `EffectPack` with `covers(&Canon)`; pure data, no
  execution; 19 to 24 tests. mesocosm-core depends on wing-glyphs (ruling
  D1) and gains `effect_pack.rs` (357) beside the experiment: `Acquiring
  { Carved, Fed, Moved }`, `MarkForm`, `PackRule` with a citation,
  `Amount` with a saturating permille curve, `MarkRequest` carrying the
  experiment glyph and no renderer type, `Refusal`, and
  `EffectPackTable::{default_pack, resolve, resolve_for_glyph,
  validate_against}`; `resolve` takes `owned` by value and no `&mut`; the
  revision pin keys the lookup on the current canon's effect; seven tests
  and the six move pins pass; no `World` touched. mesocosm-runtime's
  `GlyphReading` gains `owns_effect` and `acquired_by` reading the kept
  event, and the two invariant tests landed early: repeated resolution
  leaves the journey byte-identical, and a core-rejected carve yields no
  grant and no mark; 44 to 47 tests. Deviations: `validate_against`
  checks coverage, not cost equality, since three poles carry three cost
  shapes on one effect; `Amount` accepts any magnitude at any pole so
  uptake mass reaches the feeding pole (ruling 3); a sustained lifetime
  runs 8 to 16 ticks by amount with the caller retiring on a gap. The
  default effect id is `mesocosm:reshape-reference`, following the demo.
