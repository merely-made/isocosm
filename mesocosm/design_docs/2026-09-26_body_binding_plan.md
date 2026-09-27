# Body binding: one adapter for critters and tokens

**Date:** 2026-09-26

**Status, 2026-09-27:** ruled 2026-09-26, rulings 346 to 352 of the wing
design record; the shape is documented, not built. Mark ruled "Document the
shape only" (346): the shape in §4 goes into conatus's docs now, saying
where the module will live and how it works once built, through the text in
Appendix A, which a mere lane applies alongside the query-refresh call
(352). Nothing is built, and each product keeps its own table until he
says. The assessment written under ruling 324 on 2026-09-26 stands below as
the evidence, its forks marked with their rulings (§7).

**Owns:** the shape of conatus's body binding module, which keeps a
consumer's own key against conatus bodies (the binding table) and turns
accepted state into body changes (the accepted-map mirror); the text that
puts the shape into mere's docs (Appendix A); what each consumer changes,
and what retires, once it is built; and the done-conditions for documenting
it now and building it later.

**Does not own:** conatus, or any edit to mere, which a mere lane makes from
Appendix A; the query-refresh call, a mere edit of its own (352); the
terrain collider and its edits, which are T2's, a lane in the conatus engine
plan's §2 (rulings 330 to 335) built after the pre.4 migration (363);
pointer picking, which is isometer's alone (347); the sim's entities and
places (the [sim plan](2026-09-22_sim_plan.md)); what a token or a critter
means (each product's); or naming.

**Consumes:** the [wing design record](2026-09-18_wing_design_plan.md)
rulings 233, 299, 324, 326, 330 to 335, 346 to 352 and 363, and §4.1; the
retired
[runtime profile plan](../../design_docs/archive_docs/2026-09-18/2026-08-23_runtime_profile_plan.md);
the [engine review](2026-08-18_engine_ecology_rulings_and_review.md) §5 T1;
the [Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md) and the
[VTT overlay plan](../../design_docs/2026-09-25_vtt_overlay_plan.md); the
[board-on-isometer plan](../../design_docs/2026-09-15_board_on_isometer_plan.md);
mere's `design_docs/mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md`
and `2026-08-23_runtime_composition_acceptance_plan.md`.

Code is cited at this branch's base, `8a6a0cb`, except the retired crate,
cited at `f15fd43`. conatus is cited at the products' pin, mere `876320fd`;
every conatus line cited here reads the same at mere's `origin/main`,
`a464dc2a`, where Appendix A cites mere's docs.
*(2026-09-27: the products pin mere `0418391f` since the pin bump; every
conatus line cited here reads the same at `876320fd` and at `a464dc2a`.)*

---

## 1. What exists

**The VTT's binding table and mirror** (`crates/isometry-runtime/src/lib.rs`
at `f15fd43`, 470 lines with tests). `IsometryRuntimeProfile` owns a
conatus `Engine` with zero gravity (`:100-112`) and a
`BTreeMap<TokenSourceId, BodyId>` (`:96`). The key is map-qualified,
`TokenSourceId { map: MapSourceId(String), token: TokenId }` (`:35-52`),
because a `TokenId` is only unique within one map. `sync_accepted_map`
(`:136-229`) takes a whole accepted `MapDocument` and no intent: it builds
the desired set first and refuses a duplicate or out-of-bounds token before
touching conatus (`:142-152`), moves an existing body with `set_transform`
so its `BodyId` survives (`:163-168`), spawns a fixed cuboid for a new
token (`:170-181`), despawns what left the map (`:185-201`), then publishes
with `advance(0)`, which steps nothing (`:203-206`), and names the frame's
bodies back by key through a reverse map rebuilt each call (`:207-221`).
Tiles lower to world units through its own `IsometrySpatialConfig`
(`:54-72`, tile span 1.0, elevation step 0.25) and facing to a quaternion
(`:232-265`). Four tests cover materialize-once and silence, a move keeping
its body, a rejected event never arriving, and map qualification
(`:366-469`). A second copy of the table keyed the GPU position plane by
body slot (`resident.rs:38-43`, `:139-172`) for the product's own renderer
(`render.rs`), which ruling 299 retired with the crate.

**Mesocosm's `TactileWorld`** (`mesocosm/crates/mesocosm-runtime/src/tactile.rs`,
478 lines). It owns a conatus `BodyWorld` directly, not an `Engine`
(`:96-102`, `:118`). Create: `from_profile` builds one fixed voxel collider
from a `GroundVoxelProfile` (`:104-139`). Terrain edits: `sync` applies a
profile update's occupancy edits and refuses a stale source revision before
any edit (`:159-206`). Critters: a `BTreeMap<u64, BodyId>` under a
caller-owned `u64` (`:100`); `set_critter` despawns the old body and spawns
a new fixed body of capsule colliders, so every set mints a new `BodyId`
(`:208-235`); `clear_critter` removes (`:237-244`). Query: `pick` raycasts
and names a ground cell or a critter key, the key found by scanning the
map (`:246-279`). Because conatus's query structure only learns of new
colliders during a step, every topology change is followed by a
`step(1e-6)` (`:141-153`). Replay is not in the type: its one consumer, the
`t1_picking` example, runs the whole judgment twice from freshly grown
worlds and compares (`mesocosm-genet/examples/t1_picking/scenario.rs:181-182`),
receipted as `fnv1a64:f66edcef9ac2d3ac` (engine review §5 T1). Nothing
else constructs one; the key it uses is the constant 7 (`scenario.rs:38`).

**What conatus says about both.** `Engine` keeps "durable source bindings"
outside itself (`engine.rs:139-140`); only authorized product code lowers
accepted consequences into commands (`command.rs:29-31`); a `BodyId` is
generational, so a reused slot never answers to an old id (`body.rs:11-17`).
mere's conatus engine plan puts the table in the product's runtime profile,
"not to Conatus", because one source may become bodies, scene instances,
resident slots and audio voices, the shared minimum to be extracted "when
Paredros or Mesocosm needs the same vocabulary" (lines 186-196); binding
contracts wait for that second consumer (lines 361-367).

## 2. The common shape, and where they differ

Both keep a consumer's key against a generational `BodyId` outside conatus,
are fed only state that authority already accepted, hold fixed bodies that
never simulate, and answer in the consumer's key rather than a `BodyId`.
The retired plan's R3 named five dimensions a second consumer must test;
the two agree on authorization and differ on the other four, and on four
mechanics besides.

| | VTT mirror (retired) | `TactileWorld` |
| --- | --- | --- |
| Key | map-qualified struct | bare caller `u64` |
| Cadence | per accepted map event | per host call |
| Authorization | after the event log accepts | after the host decides; advice only |
| Subsystems | bodies only | terrain voxels and bodies |
| What it publishes | a changed/removed frame by key | pick answers by key |
| Input | the whole accepted set, reconciled | one key at a time |
| A move | `set_transform`, body kept | despawn and respawn |
| Shape | one cuboid from its config | caller's capsules |
| conatus surface | `Engine`, `advance(0)` | `BodyWorld`, settle step |

Two facts from after both were written change what the adapter is for.
isometer now picks a drawn body or the ground under a pixel, keyed by a
host-minted `SubjectKey(u64)` (`shared/isometer/src/bodies.rs:31-37`,
`query.rs:254-303`; terrain hit landed 2026-09-15), and the VTT's scene
board already routes its picks through it by `TokenId`
(`crates/isometry-views/src/scene/board.rs:177-200`). So pointer picking,
`TactileWorld`'s one use, no longer needs a conatus binding. And the
board-on-isometer plan's "tokens as live bodies" (its I4) means tokens drawn
as isometer bodies, which landed; it is not tokens as conatus bodies. No
code in either product queries a bound conatus body today. The VTT's line
of sight is grid arithmetic in `isometry-core` (`visibility.rs:170`), and
`mesocosm-core` stays Parry-free (engine review §5, line 561). Ruling 346
answered this: the shape is documented now and built when Mark says.

Eponym is a third shape (ruling 233 puts its motion and contact solver on
the game side over conatus): it rebuilds a `BodyWorld` from product state
for each move and keeps no table (`eponym/crates/eponym-world/src/contact/spatial.rs:5`,
`:27`; `motion.rs:158`), and it too steps before querying (`spatial.rs:52`,
`motion.rs:187`). Its per-move rebuild is a cold reconcile, which the shape
already covers (§7, fork 8).

## 3. Where it lives

**Ruled 2026-09-26 (348): a module in mere's `conatus`, generic over the
consumer's key,** built when Mark says (346). The record's §2 test puts it
in the stack: every game needs bodies by key, and keeping a key decides
nothing about the world; the record's §4.1 places bodies, collision and
queries in conatus. Every consumer already pins conatus (Mesocosm, Eponym
and isometer at `876320fd`), and a generic key keeps product types out of
it. mere's objection, that one source becomes several runtime things, now
holds for nothing but bodies: isometer carries the host's key itself
(`SubjectKey`), so scene instances need no table, and the resident plane
retired with its renderer (the record's §4.2). Mesocosm is the second
consumer mere's plan waited for before extracting the shared minimum.

The alternatives weighed were a new crate under `shared/` (a crate for a
component of a few hundred lines, against Mark's rule of 2026-09-23 that a
component is not a crate), a module in isometer (the rendering tier, which
names no conatus body), and `mesocosm-runtime`, which T1 named the
adapter's permanent host on 2026-08-26 (engine review line 344) and which
would make the VTT depend on a Mesocosm crate. Ruling 348 supersedes T1's
naming. The three mere doc passages that placed the table outside conatus
are amended with the shape (Appendix A.2).

## 4. The shape

Ruled 2026-09-26 (347 to 352). Illustrative, not compile-ready; the names
are placeholders, not proposals. Appendix A.1 states the same shape in
prose for conatus's docs.

```rust
/// A consumer's keys against conatus bodies. Holds no world, no terrain,
/// and nothing durable: rebuilt from the consumer's accepted state.
pub struct BodyBindings<K: Ord + Clone> {
    bodies: BTreeMap<K, Bound>,
    keys: BTreeMap<BodyId, K>, // kept in step, never rebuilt per call
}
struct Bound { body: BodyId, shape: u64 }

/// What the consumer lowered: where the body stands, what it is made of,
/// and the caller's own revision of that shape (a change respawns, 351).
pub struct BodySpec {
    pub transform: Transform,
    pub shape: u64,
    pub colliders: Vec<ColliderDesc>,
}

/// A respawned key (its shape changed) is listed as spawned.
pub struct Changes<K> { pub spawned: Vec<K>, pub moved: Vec<K>, pub removed: Vec<K> }

pub enum BindingError<K> {
    DuplicateKey(K),                   // in one reconcile, found before any world call
    LostBody { key: K, body: BodyId }, // despawned behind the module's back
    Body(BodyError),
}

impl<K: Ord + Clone> BodyBindings<K> {
    /// The accepted-map mirror (350): the whole accepted set. Spawn new keys,
    /// move moved ones (351), respawn a changed shape, despawn absent keys.
    /// Validated in full before the first world call; a cold rebuild is this
    /// on an empty table.
    pub fn reconcile(&mut self, world: &mut BodyWorld,
        desired: impl IntoIterator<Item = (K, BodySpec)>) -> Result<Changes<K>, BindingError<K>>;
    /// One key changed alone (350).
    pub fn set(&mut self, world: &mut BodyWorld, key: K, spec: BodySpec)
        -> Result<Changes<K>, BindingError<K>>;
    pub fn remove(&mut self, world: &mut BodyWorld, key: &K)
        -> Result<Option<BodyState>, BindingError<K>>;
    pub fn body(&self, key: &K) -> Option<BodyId>;
    /// Names any conatus answer: a `RayHit`'s body, an overlap's colliders, a
    /// `FrameUpdate`'s changed and removed ids. An unbound body, the terrain
    /// among them, is `None`, and the caller keeps the raw answer.
    pub fn key(&self, body: BodyId) -> Option<&K>;
}
```

It binds bodies only and borrows the caller's `BodyWorld` (349; an `Engine`
lends one, `engine.rs:166`), so bound bodies share one world with the
terrain collider that T2's one edit path keeps current (rulings 330 to
335). Accepted state arrives whole, as a reconcile that is also the cold
rebuild, or per key (350); cold and incremental must agree. A moved body
keeps its id, and only a new shape revision respawns it (351). Queries stay
conatus's own calls and the module names what they hit; after a topology
change, queries see it through conatus's refresh call (352), not a settle
step. Pointer picks on a drawn frame are isometer's (347). Several bodies
for one source use a compound key such as `(source, part)`. It takes no
intent and serializes no `BodyId`.

## 5. Adoption, and what retires

**Now (346).** Nothing adopts and nothing retires. Mesocosm keeps
`TactileWorld` whole: its critter table, its pick and its terrain half. The
VTT has no table, since ruling 299 retired `isometry-runtime`; if a token
needs a conatus body before the module is built, the VTT keeps its own.
Directing's click (M3) goes through isometer's pick (347). One change comes
without the bindings: when the refresh call lands (352) and a product
repins, `TactileWorld`'s settle step (`tactile.rs:141-153`) and Eponym's
pre-query steps (`contact/spatial.rs:52`, `motion.rs:187`) become that call.

**When Mark says build.** In Mesocosm, `tactile.rs`'s critter map,
`set_critter` and `clear_critter` (`:100`, `:208-244`) become the module,
keyed by `OrganismId` (`mesocosm-core/src/organism.rs:45`) until M2's bodies
family moves and the sim's `EntityHandle`
(`shared/isocosm-overlay/src/handle.rs:10`) names critters. The critter
branch of `pick` (`:264-271`) becomes `key()`. Capsule lowering
(`:303-354`) stays Mesocosm's, as the caller's shape. The `t1_picking`
example ports. The terrain half (`:104-206`) is not the module's: T2's one
path replaces it on T2's own schedule (349; rulings 330 to 335, 363).

In the VTT, once tokens become conatus bodies: after `isonetry` accepts a
map event (`crates/isonetry/src/session/apply.rs:54-59`), the host calls
`reconcile` with the map's tokens. The key stays map-qualified: `(map,
TokenId)` today, and under ruling 243 the site's `PlaceHandle` in the map's
place (`TableActKind::Move { to: Cell { site, at } }`,
`shared/isocosm-overlay/src/vtt/table.rs:37`, `:45`). Tiles lower through
the scene board's own convention, `BoardWorld::stand`
(`crates/isometry-views/src/scene/world.rs:110`), so a token's conatus body
and its drawn body stand at the same point; the retired
`IsometrySpatialConfig` does not come back. Facing lowers as
`facing_rotation` did, in the VTT.

What retires then: `TactileWorld`'s critter half, and the interim wording
Appendix A puts in conatus's docs ("documented, not built", and the
`Engine` comment's "until the binding module is built"). The rest of
`crates/isometry-runtime` retired with ruling 299 (its deletion is another
branch's); its table and mirror are carried by the module, and its second
table, resident plane and renderer have no successor, because a product
owns no renderer (the record's §4.2).

## 6. Done-conditions

**Documented, now (346).** Done when a mere lane has applied Appendix A at
mere's main: the "Body bindings" text of A.1 stands in the conatus engine
plan's §1; the three passages read as A.2 gives them; mere's two index rows
carry A.3's sentences, as mere's DOC_POLICY §6 asks; no binding code is
written, the `Engine` doc comment being the only line under `crates/` the
change touches; no live mere passage still puts the key-to-body table
outside conatus, those A.4 lists saying only that a source's meaning is the
profile's; and this plan's status and progress name the mere commit. The
refresh call (352) may ride in the same lane as a change of its own, with
its own tests.

**Built, when Mark says.** Each receipt that draws is a seeded draw, not a
fixture (ruling 15).

- **Build.** Done when the module exists in `conatus`, names no product
  type, adds no dependency to conatus, and stays under the 600-line
  ceiling; over a seeded draw of accepted sets and edits, a cold
  `reconcile` and the same history applied through `set` and `remove`
  leave the same keys, transforms and shapes; a key removed and added back
  never resolves to its old body, and an old `BodyId` resolves to no key; a
  duplicate key, and a body despawned behind the module, are refused before
  any world call; after a conatus refusal partway through an apply, the
  table names exactly the bodies the world holds; `key()` of an unbound
  body is `None`; no `BodyId` reaches anything serialized; and conatus's
  docs drop their interim wording in the same change.
- **Mesocosm adopts.** Done when `TactileWorld`'s critter tests pass through
  the module and `t1_picking` reproduces its receipt: 22 judged stops, 16
  ground, 4 critter, 2 nothing, the judgment agreeing bit for bit across two
  fresh runs.
- **The VTT adopts,** when its first conatus consumer opens. Done when the
  retired profile's four tests (`lib.rs:366-469`), ported to the board's
  lowering, pass against the module, and a token's conatus body and its
  drawn body stand at the same world point for every tile and elevation of
  a seeded draw of maps.
- In every workspace the change touches,
  `cargo check --workspace --all-features --all-targets` is green.

## 7. Forks, as ruled

Put to Mark on 2026-09-26 as eight forks; he ruled seven the same day.

1. **The premise had shifted** (§2). Asked: design the shared mechanism
   anyway (recommended), document the shape only, or retire
   `TactileWorld`'s critter half. **Ruling 346.** Mark: "Document the
   shape only." Asked how that sits with 348 and 349, he confirmed the
   reading: the common shape goes into conatus's docs now, saying where the
   module will live and how it works once built; nothing is built, and each
   product keeps its own table until he says.
2. **Pointer picking.** Asked: isometer only (recommended), conatus through
   the bindings as T1 was ruled, or both. **Ruling 347.** Mark: "isometer
   only." The player clicks what was drawn and isometer answers per part and
   per frame, as the VTT board already does; the bindings answer other
   queries.
3. **Where it lives.** Asked: a module in mere's conatus (recommended), a
   new crate under `shared/`, isometer, or `mesocosm-runtime`. **Ruling
   348.** Mark: "A module in mere's conatus." It is generic over the key,
   and three mere doc passages are amended with the shape, one of them the
   line placing bindings "not to Conatus" (Appendix A.2).
4. **Terrain and the world.** Asked: bodies only, on a borrowed world shared
   with T2's terrain collider (recommended), bodies only on its own world,
   or bodies and terrain. **Ruling 349.** Mark: "Bodies only, shared
   world." Terrain arrives through T2's one path (rulings 330 to 335).
5. **How accepted state arrives.** Asked: both a whole-set reconcile and
   per-key set and remove (recommended), reconcile only, or per key only.
   **Ruling 350.** Mark: "Both." The reconcile is the cold rebuild and
   carries the VTT's per-event accepted map; per-key updates carry a critter
   changing alone; cold and incremental must agree.
6. **A pose change.** Asked: move the body and keep its id, respawning only
   when the shape revision changes (recommended), or always respawn.
   **Ruling 351.** Mark: "Move it, keep its id."
7. **Refreshing conatus's queries.** Asked: keep a settle step in the
   adapter, a refresh call in conatus (recommended), or leave it to the
   world's owner. **Ruling 352.** Mark: "A refresh call in conatus." It is
   a small mere edit, independent of the bindings, that `TactileWorld`'s
   settle step and Eponym's pre-query steps both become.
8. **Eponym.** Not put to Mark: with nothing built, whether Eponym is in the
   build is moot. Its per-move rebuild (`contact/spatial.rs:27`,
   `motion.rs:158`) is a cold reconcile, which the shape already covers.

## Findings

- **2026-09-26:** mere's composition plan records the tactile-bodies row as
  "Met 2026-08-26 by Mesocosm", the contract "held by Conatus's public
  vocabulary" (line 52), while its C4 gate for the same second-consumer
  challenge still reads "Open" (line 171). A.2's third replacement
  reconciles the two.
- **2026-09-26:** `TactileWorld` has one caller, the `t1_picking` example;
  the headed Mesocosm host builds no tactile world (dev tools plan, finding
  3 at line 248).
- **2026-09-26:** the retired mirror validated its whole desired set before
  mutating (`lib.rs:142-152`), but a conatus refusal partway through its
  apply loop would leave table and world half-applied, and neither version
  says what that leaves; the build's done-condition now does.
- **2026-09-26:** `crates/isometry-runtime` is still on main and excluded
  from the root workspace (`Cargo.toml:13`); its deletion under ruling 299
  is on another branch.
- **2026-09-27:** at mere `a464dc2a` the conatus engine plan's §2 (lines
  204-231) does not yet hold the T2 lane ruling 335 puts there, so A.1
  points to T2 by its rulings rather than by a heading.
- **2026-09-27:** the composition plan's Isometry profile section (lines
  68-97 at `a464dc2a`) still describes `isometry-runtime` as a live slice;
  ruling 299 retired it. It places no binding outside conatus, so Appendix
  A leaves it; reported for the mere lane.

## Progress

- **2026-09-26:** assessment drafted under ruling 324 from the retired
  crate at `f15fd43`, `TactileWorld`, conatus at `876320fd`, isometer's
  picks, the overlay contract and both overlay plans. Nothing built; eight
  forks for Mark.
- **2026-09-27:** Mark ruled seven forks on 2026-09-26 (rulings 346 to
  352), the eighth moot. The plan is revised to the rulings (status, §3 to
  §7), and Appendix A carries the text for conatus's docs, cited at mere
  `origin/main`, `a464dc2a`; rebased onto `8a6a0cb`. Nothing built, and
  mere untouched.

---

## Appendix A. Text for conatus's docs

For a mere lane to apply at mere's main, alongside the query-refresh call
(352). Everything is cited at mere `origin/main`, `a464dc2a`. Each
replacement quotes the text it replaces, so it can be applied by match if
lines have moved.

### A.1 Add to the conatus engine plan

In `design_docs/mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md`,
§1 "Runtime and systems", after the paragraph A.2's second item replaces
(lines 186-196) and before "The remaining runtime work is" (line 198):

```markdown
**Body bindings (ruled 2026-09-26; documented, not built).** A product keeps
its own key against the Conatus bodies that stand for its sources, such as
a VTT token or a Mesocosm critter. Isometry's retired accepted-map profile
and Mesocosm's `TactileWorld` built that table alike; Isometry's body
binding plan compares them
(`repos/isometry/mesocosm/design_docs/2026-09-26_body_binding_plan.md`).
Mark ruled its shape documented here now and its code built later (wing
design record, rulings 346 to 352,
`repos/isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`).
Until he says to build it, each product keeps its own table: Mesocosm's in
`mesocosm-runtime`'s `TactileWorld`, while the VTT, whose `isometry-runtime`
retired, has none today.

When built, it is a module in `conatus`, generic over the product's key
(ruling 348), and works like this:

- **Bodies only, on the caller's world** (349). It owns no `BodyWorld`; each
  call borrows the caller's, which an `Engine` lends through `bodies_mut`,
  so bound bodies share one world with the terrain collider that T2's one
  edit path keeps current (rulings 330 to 335). It holds no terrain, no
  clock and nothing durable, and never serializes a `BodyId`.
- **The table.** Each key maps to one `BodyId` and the shape revision it was
  built from, with the reverse map kept in step. A key means what the
  product says: a map-qualified token id for the VTT, an organism or entity
  id for a critter. Several bodies for one source use a compound key such
  as `(source, part)`.
- **Accepted state, whole or per key** (350). A reconcile takes the whole
  accepted set: it spawns new keys, moves moved ones and despawns absent
  ones. It is the cold rebuild, and it carries the VTT's accepted map on
  each event. Per-key set and remove carry one source changing alone, such
  as a critter. Cold and incremental must agree. Only authorized product
  code calls it, with state its authority has already accepted; it takes no
  intent.
- **A moved body keeps its id** (351). A new pose moves the body. Only a
  change of the caller's shape revision respawns it.
- **Refusals before mutation.** A key twice in one reconcile, and a bound
  body despawned behind the module, are refused before any world call.
  After a Conatus refusal partway through an apply, the table names exactly
  the bodies the world holds.
- **Queries stay Conatus's.** Rays, overlaps, character moves and frames are
  Conatus's own calls; the module names what they touch by key, and a body
  it does not bind, the terrain among them, has none. After a topology
  change a query sees it through Conatus's query-refresh call (352), not a
  settle step. Pointer picks on a drawn frame are isometer's (347).

What a key means, when a source binds, and what else it becomes (scene
instances, resident slots, audio voices) stay the product's. Eponym's
per-move query world is a cold reconcile, so the shape covers it without
change. The conditions for building the module are in the body binding
plan's §6.
```

### A.2 Replace the three passages that place bindings outside conatus

**1. conatus's `Engine` doc comment,** `crates/conatus/conatus/src/engine.rs:139-140`.

Replace:

```rust
/// [`FrameUpdate`] to its selected consumers. Rules, input, audio, rendering,
/// and durable source bindings remain outside this runtime.
```

With:

```rust
/// [`FrameUpdate`] to its selected consumers. Rules, input, audio, rendering,
/// and what a product's sources mean remain outside this runtime. Until the
/// binding module is built (conatus engine plan §1, "Body bindings"), a
/// product keeps its own keys against bodies.
```

**2. The engine plan's "Source bindings stay profile-owned",**
`design_docs/mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md:186-196`.

Replace:

```markdown
**Source bindings stay profile-owned.** `BodyId` is generational and
runtime-only, and `BodyDesc` deliberately carries no durable source
reference. One durable product source may materialize as several bodies,
scene instances, resident slots, and audio voices, so the binding table
(`ProductSourceId -> RuntimeBindings { bodies, scene instances, resident
slots, audio voices }`) belongs to the runtime profile, not to Conatus and
not inside `BodyDesc`. Sceno's `SourceRef` is the pattern reference, not
automatically the universal type: it belongs to semantic scenes and lacks
revision and materialization information. The first Isometry profile
defines a neutral-shaped binding table locally; the shared minimum is
extracted when Paredros or Mesocosm needs the same vocabulary.
```

With:

```markdown
**What a source means stays profile-owned; its body table will be a Conatus
module.** `BodyId` is generational and runtime-only, and `BodyDesc`
deliberately carries no durable source reference. One durable product
source may materialize as several bodies, scene instances, resident slots,
and audio voices, so what a source is, when it binds, and what it becomes
belong to the runtime profile, not to Conatus and not inside `BodyDesc`.
Sceno's `SourceRef` is the pattern reference, not automatically the
universal type: it belongs to semantic scenes and lacks revision and
materialization information. The first Isometry profile defined a
neutral-shaped binding table locally, and Mesocosm's `TactileWorld` needed
the same vocabulary. On 2026-09-26 Mark ruled the shared minimum, the table
keeping a product's own key against its bodies, a Conatus module generic
over the key, documented below as "Body bindings" and built when he says
(wing design record, rulings 346 and 348); until then each product keeps
its own table.
```

**3. The composition plan's tactile row and C4,**
`design_docs/mere_docs/implementation_strategy/2026-08-23_runtime_composition_acceptance_plan.md:52`
and `:171-180`.

Replace the row at line 52:

```markdown
| Tactile bodies and spatial queries | Conatus owns the state it advances | Product source bindings to runtime `BodyId`s | Isometry product profile mirrors accepted map tokens; Mesocosm's `mesocosm-runtime` tactile adapter (terrarium picking, T1 2026-08-26) is the second, oracle-judged consumer | Met 2026-08-26 by Mesocosm; the shared vocabulary stays Conatus ids and arrays, Rapier private | Two profiles proven; contract held by Conatus's public vocabulary |
```

With:

```markdown
| Tactile bodies and spatial queries | Conatus owns the state it advances, and will own the table keeping a product's key against its bodies, as a module generic over the key | The product's own keys, and what they mean, against runtime `BodyId`s | Isometry product profile mirrors accepted map tokens; Mesocosm's `mesocosm-runtime` tactile adapter (terrarium picking, T1 2026-08-26) is the second, oracle-judged consumer; the two compared on C4's five dimensions 2026-09-26 | Ruled 2026-09-26 (wing design record, rulings 346 to 352): the shape is documented in the conatus engine plan §1, "Body bindings", and built when Mark says; pointer picks are isometer's (347) | Shape documented, not built; each product keeps its own table |
```

Replace C4, lines 171-180:

```markdown
### C4 — Second consumer challenge

**Open.** Paredros or Mesocosm consumes the relevant Isometry profile shape and
forces at least one real comparison of source identity, trigger cadence,
authorization, subsystem selection, and frame consumption. Paredros's
nonspatial F3 proof may establish cold rebuild versus incremental delta,
recipe-version refusal, removal-before-replacement, and one durable source
materializing into several runtime bindings, but it cannot close C4 without an
embodied spatial consumer. Only the common minimum may then move to a shared
contract. Product-specific fields and epistemic vocabulary remain local.
```

With:

```markdown
### C4 — Second consumer challenge

**Compared 2026-09-26; the shape is documented, not built.** Mesocosm's
`TactileWorld` (T1, 2026-08-26) is the embodied spatial consumer this gate
asked for. Isometry's body binding plan
(`repos/isometry/mesocosm/design_docs/2026-09-26_body_binding_plan.md` §2)
compared it with the retired Isometry profile on source identity, trigger
cadence, authorization, subsystem selection, and frame consumption: they
agree on authorization and differ on the other four. Mark ruled the common
minimum, the table keeping a product's own key against its bodies, a
Conatus module generic over the key, documented in the conatus engine
plan's §1 and built when he says (wing design record, rulings 346 to 352).
The comparison is made; the minimum moves when the module is built, and
until then each product keeps its own table. Paredros's nonspatial F3 proof
may still establish cold rebuild versus incremental delta, recipe-version
refusal, removal-before-replacement, and one durable source materializing
into several runtime bindings. Product-specific fields and epistemic
vocabulary remain local.
```

### A.3 mere's index rows

mere's DOC_POLICY §6 has `design_docs/DOC_README.md` change with the docs it
indexes. Append a sentence to each row:

- Line 94, the conatus engine plan's row, after "and the Nexus
  decomposition.": "Body bindings ruled 2026-09-26 (wing design record,
  rulings 346 to 352): documented in §1 as a Conatus module generic over the
  key, built when Mark says; each product keeps its own table meanwhile."
- Line 95, the composition plan's row, after "cross-product identity
  contracts stay gated.": "Advanced 2026-09-26: C4's comparison is made and
  the tactile row's shared minimum ruled a Conatus module, documented and
  not built (rulings 346 to 352)."

### A.4 Passages that stand as written

These name source bindings as the profile's and stay true once A.2 is
applied, because the profile still decides what a key means, what binds and
when; the module is a mechanism it calls.

- The conatus engine plan, lines 30-37 and 57-61: the profile conducts
  clocks, triggers, input mapping, authorization, source bindings and
  subsystem selection.
- The same plan, lines 48-49: the 2026-08-23 correction "Source bindings are
  profile-owned". It is a dated ruling list, kept as history, and its
  "(§1)" now leads to A.2's text.
- The same plan, line 249: the product owns the resident plane's source
  bindings. Resident slots are outside this shape.
- The same plan, lines 361-367: contracts wait for the second consumer. It
  is the rule rulings 346 and 348 satisfied.
- The composition plan, lines 17-20 and 49: a profile chooses its source
  bindings, and owns them for semantic projection; and lines 191-192: a
  first consumer cannot establish a source-binding schema.
