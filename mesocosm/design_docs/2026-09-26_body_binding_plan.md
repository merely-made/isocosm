# Body binding: one adapter for critters and tokens

**Date:** 2026-09-26

**Status, 2026-09-26:** assessment, for Mark's forks (§7). Nothing is built
and nothing in mere is touched. Written under wing design record ruling 324
("Plan it now"): one binding adapter serving Mesocosm's critters and the
VTT's tokens, taking the retired `isometry-runtime` pieces (readable at
`f15fd43`) and Mesocosm's `TactileWorld` as input, built when Mark says.

**Owns:** the design of one adapter that keeps a consumer's own key against
conatus bodies (the binding table) and turns accepted state into body
changes (the accepted-map mirror); where it lives; what each consumer
changes to adopt it; and the done-conditions for building it.

**Does not own:** conatus itself or any edit to mere; the terrain collider
and its edits (the functional loops plan's T2, being assessed under ruling
326); pointer picking on a drawn frame (isometer's, `shared/isometer/src/query.rs`);
the sim's entities and places (the [sim plan](2026-09-22_sim_plan.md));
what a token or a critter means (each product's); or naming.

**Consumes:** the [wing design record](2026-09-18_wing_design_plan.md)
rulings 233, 299, 324 and 326 and §4.1; the retired
[runtime profile plan](../../design_docs/archive_docs/2026-09-18/2026-08-23_runtime_profile_plan.md);
the [engine review](2026-08-18_engine_ecology_rulings_and_review.md) §5 T1;
the [Mesocosm overlay plan](2026-09-25_mesocosm_overlay_plan.md) and the
[VTT overlay plan](../../design_docs/2026-09-25_vtt_overlay_plan.md); the
[board-on-isometer plan](../../design_docs/2026-09-15_board_on_isometer_plan.md);
mere's `design_docs/mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md`
and `2026-08-23_runtime_composition_acceptance_plan.md`.

Code is cited at this branch's base, `aa0fe2b`, except the retired crate,
cited at `f15fd43`. conatus is cited at the products' pin, mere `876320fd`;
every conatus line cited here reads the same in mere's local checkout of
main, `059c1855`.

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
`mesocosm-core` stays Parry-free (engine review §5, line 561). §7 fork 1
takes this to Mark.

Eponym is a third shape (ruling 233 puts its motion and contact solver on
the game side over conatus): it rebuilds a `BodyWorld` from product state
for each move and keeps no table (`eponym/crates/eponym-world/src/contact/spatial.rs:5`,
`:27`; `motion.rs:158`), and it too steps before querying (`spatial.rs:52`,
`motion.rs:187`).

## 3. Where it lives

The record's §2 test puts it in the stack: every game needs bodies by key,
and keeping a key decides nothing about the world. The record's §4.1
places bodies, collision and queries in conatus. The candidates:

- **A module in mere's `conatus` crate, generic over the consumer's key.**
  Every consumer already pins conatus (Mesocosm, Eponym and isometer at
  `876320fd`). Generic `K` keeps product types out of conatus, and the
  meaning of a key stays with its owner. mere's objection was that one
  source becomes several runtime things; since then isometer carries the
  host's key itself (`SubjectKey`), so scene instances need no table, and
  the resident plane retired with its renderer (the record's §4.2), leaving
  bodies, which are conatus's; and Mesocosm is the second consumer that plan
  was waiting for. Cost: a mere edit, and three mere doc passages amended
  (`engine.rs:139-140`; the engine plan's lines 186-196; the composition
  plan's row at line 52 and its C4 at line 171).
- **A new crate under `shared/`.** No mere edit, but a crate for a component
  of a few hundred lines, against Mark's rule of 2026-09-23 that a
  component is not a crate, and a name through a naming round.
- **A module in isometer.** isometer holds the drawn half of the key, but it
  is the rendering tier and names no conatus body (ruling 324); this would
  make it the physics tier's host too.
- **Stay in `mesocosm-runtime`,** which T1 named the adapter's permanent host
  on 2026-08-26 (engine review line 344). The VTT would then depend on a
  Mesocosm product crate, which fails the record's §2 test.

Recommended: the conatus module (§7 fork 3).

## 4. Interface sketch

Illustrative, not compile-ready. The names are placeholders, not proposals.

```rust
/// A consumer's keys against conatus bodies. Holds no world, no terrain,
/// and nothing durable: rebuilt from the consumer's accepted state.
pub struct BodyBindings<K: Ord + Clone> {
    bodies: BTreeMap<K, Bound>,
    keys: BTreeMap<BodyId, K>, // kept in step, never rebuilt per call
}
struct Bound { body: BodyId, shape: u64 }

/// What the consumer lowered: where the body stands, what it is made of,
/// and the caller's own revision of that shape (a change respawns).
pub struct BodySpec {
    pub transform: Transform,
    pub shape: u64,
    pub colliders: Vec<ColliderDesc>,
}

/// A respawned key (its shape changed) is listed as spawned.
pub struct Changes<K> { pub spawned: Vec<K>, pub moved: Vec<K>, pub removed: Vec<K> }

pub enum BindingError<K> {
    DuplicateKey(K),                  // in one reconcile, found before any world call
    LostBody { key: K, body: BodyId }, // despawned behind the adapter's back
    Body(BodyError),
}

impl<K: Ord + Clone> BodyBindings<K> {
    /// The accepted-map mirror: the whole accepted set. Spawn new keys, move
    /// moved ones, respawn a changed shape, despawn absent keys. Validated in
    /// full before the first world call; a cold rebuild is this on an empty table.
    pub fn reconcile(&mut self, world: &mut BodyWorld,
        desired: impl IntoIterator<Item = (K, BodySpec)>) -> Result<Changes<K>, BindingError<K>>;
    /// One key changed.
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

It takes a borrowed `BodyWorld` (an `Engine` lends one, `engine.rs:166`),
so terrain, bound bodies and anything unbound share one world (fork 4).
Queries stay conatus's own calls; the adapter only names what they hit.
Several bodies for one source use a compound key such as `(source, part)`,
which reserves room for articulated or dynamic bodies without a second
table. It takes no intent and serializes no `BodyId`.

## 5. Adoption, and what retires

**Mesocosm.** In `tactile.rs`, the critter map, `set_critter` and
`clear_critter` (`:100`, `:208-244`) become the adapter, keyed by
`OrganismId` (`mesocosm-core/src/organism.rs:45`) until M2's bodies family
moves and the sim's `EntityHandle` (`shared/isocosm-overlay/src/handle.rs:10`)
names critters. The critter branch of `pick` (`:264-271`) becomes `key()`.
Capsule lowering (`:303-354`) stays Mesocosm's, as the caller's shape. The
terrain half (`:104-206`) stays until T2 decides the terrain collider's
home (fork 4). The `t1_picking` example ports. Directing's click (M3) goes
through isometer's pick unless fork 2 says otherwise.

**The VTT, when tokens become conatus bodies.** After `isonetry` accepts a
map event (`crates/isonetry/src/session/apply.rs:54-59`), the host calls
`reconcile` with the map's tokens. The key stays map-qualified: `(map,
TokenId)` today, and under ruling 243 the site's `PlaceHandle` in the
map's place (`TableActKind::Move { to: Cell { site, at } }`,
`shared/isocosm-overlay/src/vtt/table.rs:37`, `:45`). Tiles lower through
the scene board's own convention, `BoardWorld::stand`
(`crates/isometry-views/src/scene/world.rs:110`), so a token's conatus body
and its drawn body stand at the same point; the retired
`IsometrySpatialConfig` does not come back. Facing lowers as
`facing_rotation` did, in the VTT.

**What retires.** The rest of `crates/isometry-runtime` (ruling 299; its
deletion is another branch's): its table and mirror are carried here; its
second table, resident plane and renderer have no successor, because a
product owns no renderer (the record's §4.2). `TactileWorld`'s critter
half. T1's naming of `mesocosm-runtime` as the adapter's permanent host, if
fork 3 moves it. The three mere doc passages in §3 are amended, if the
module goes to conatus.

## 6. Done-conditions for building it

One build phase, then two adoptions. Each receipt that draws is a seeded
draw, not a fixture (ruling 15).

- **Build.** Done when the adapter exists where fork 3 puts it, names no
  product type, adds no dependency beyond its host crate's, and stays under
  the 600-line ceiling; over a seeded draw of accepted sets and edits, a
  cold `reconcile` and the same history applied through `set` and `remove`
  leave the same keys, transforms and shapes; a key removed and added back
  never resolves to its old body, and an old `BodyId` resolves to no key; a
  duplicate key, and a body despawned behind the adapter, are refused before
  any world call; after a conatus refusal partway through an apply, the
  table names exactly the bodies the world holds; `key()` of an unbound body
  is `None`; and no `BodyId` reaches anything serialized.
- **Mesocosm adopts.** Done when `TactileWorld`'s critter tests pass through
  the adapter and `t1_picking` reproduces its receipt: 22 judged stops, 16
  ground, 4 critter, 2 nothing, the judgment agreeing bit for bit across two
  fresh runs.
- **The VTT adopts,** when its first conatus consumer opens. Done when the
  retired profile's four tests (`lib.rs:366-469`), ported to the board's
  lowering, pass against the adapter, and a token's conatus body and its
  drawn body stand at the same world point for every tile and elevation of
  a seeded draw of maps.
- In every workspace the change touches,
  `cargo check --workspace --all-features --all-targets` is green.

## 7. Forks for Mark

1. **The premise has shifted.** The question put for ruling 324 named the
   VTT's tokens as live bodies as a second consumer; the board-on-isometer
   plan's live bodies are drawn isometer bodies, landed and keyed by
   `TokenId`, and pointer picking, T1's one use, is now isometer's. No code
   queries a bound conatus body today.
   (a) Design the adapter anyway as the shared mechanism, with the ported
   receipts as its first consumers and the first non-pointer query (a ray
   that is not the camera's, an overlap, contact) as its first live one.
   (b) Write the common shape into conatus's docs and let each product keep
   its own table, which is where mere's docs stand today. (c) Retire
   `TactileWorld`'s critter half with `isometry-runtime` and let Eponym's
   solver found the table when it first needs one. *Recommended: (a).* The
   mechanism is the same in both versions, the known consumers (tokens
   standing in a volume battlemap under ruling 243, critters in the
   terrarium, Eponym's solver under ruling 233) all need a body by key, and
   the design costs nothing until Mark says build. *That the first two need
   conatus queries is a reading, not ruled.*
2. **Pointer picking.** (a) isometer's drawn-frame pick only
   (`query.rs:271`), and the adapter answers other queries. (b) conatus
   through the adapter, as T1 was ruled. (c) Both: isometer for the player's
   click, conatus for a headless pick on the bench. *Recommended: (a).* The
   player clicks what was drawn, isometer answers per part and per frame,
   and the VTT board already routes through it (`board.rs:177-200`); a
   bench pick can ray conatus and call `key()` without a picking API.
3. **Where it lives.** (a) A module in mere's `conatus`, generic over the
   key. (b) A new crate under `shared/`. (c) A module in isometer. (d)
   `mesocosm-runtime`, as T1 named. *Recommended: (a).* The record's §4.1
   places bodies and queries in conatus, every consumer already pins it,
   and generic keys keep product meaning out; it costs a mere edit and the
   three mere doc passages in §3. (b) mints a crate for a component, (c)
   puts physics in the rendering tier, and (d) makes the VTT depend on a
   Mesocosm crate.
4. **Terrain and the world.** (a) Bodies only, on a borrowed `BodyWorld`
   that T2's terrain collider shares. (b) Bodies only, on a world the
   adapter owns. (c) Bodies and terrain on its own world, `TactileWorld`
   whole, with `GroundVoxelProfile` (`mesocosm-core/src/voxel_profile.rs`,
   which needs only isometer-core's `Ground` and nisus) moving beside it.
   *Recommended: (a).* T2 is one edit reaching every spatial consumer,
   colliders included, and is being assessed (ruling 326); a terrain feed
   here would be a second path for that edit, the duplication the record's
   §4.3 already flags, and an owned world pushes anything unbound into a
   second one.
5. **How accepted state arrives.** (a) Both a whole-set `reconcile` and
   per-key `set`/`remove`, reconcile being the cold rebuild. (b) Reconcile
   only. (c) Per-key only. *Recommended: (a).* The VTT's source is a whole
   accepted map per event (`lib.rs:136-229`) and a critter changes alone;
   replay needs the cold path, and the retired R3 asked that cold and
   incremental agree.
6. **A pose change.** (a) Move the body and keep its `BodyId`; respawn only
   when the caller's shape revision changes. (b) Always respawn, as
   `TactileWorld` does. *Recommended: (a).* Ids are generational
   (`body.rs:11-17`), so a respawn per move churns every id anything holds
   between frames, such as an interaction event's colliders; (b) is simpler
   and harmless only while nothing steps. The sim's `Entity` already carries
   a `body_revision` (`shared/isocosm/src/schema.rs:63`) for the caller to
   pass.
7. **Refreshing conatus's queries.** (a) The adapter settles with a minimal
   step after each topology change, as `TactileWorld` does. (b) conatus
   gains a query-refresh call, a mere edit, which both `TactileWorld`'s
   settle and Eponym's pre-query steps become. (c) Leave refresh to the
   world's owner. *Recommended: (b).* Two consumers already work around its
   absence (`tactile.rs:141-153`, `spatial.rs:52`), and `TactileWorld`'s
   comment names this as the call it replaces. `advance(0)` does not step
   (`engine.rs:259`), so the retired mirror's frame would not have
   refreshed queries either; that is read from the code, not run.
8. **Eponym.** (a) Out of this build; its per-move rebuild is a cold
   `reconcile`, so the interface reserves room with no change. (b) In, as
   the third consumer and the first with a live query. *Recommended: (a).*
   Ruling 324 names two consumers, and Eponym's solver belongs to its own
   plan; widening is Mark's call, and (b) is the natural first live
   consumer if fork 1's (a) wants one sooner.

## Findings

- **2026-09-26:** mere's composition plan records the tactile-bodies row as
  "Met 2026-08-26 by Mesocosm", the contract "held by Conatus's public
  vocabulary" (line 52), while its C4 gate for the same second-consumer
  challenge still reads "Open" (line 171). Reported, not changed; mere is
  read-only here.
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

## Progress

- **2026-09-26:** assessment drafted under ruling 324 from the retired
  crate at `f15fd43`, `TactileWorld`, conatus at `876320fd`, isometer's
  picks, the overlay contract and both overlay plans. Nothing built; eight
  forks for Mark.
