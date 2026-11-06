# Uniform bricks

**Status:** 2026-11-05: plan. Nothing built. D1 to D4 answered.

**For:** `Ground` (`shared/isometer/crates/isometer-core/src/ground.rs`)
growing a brick at a time, keeping a brick of one material as that material,
and encoding such a brick compactly; and measuring how often Mesocosm clones
the ground. The representation is owed onward to nisus when the store moves
there (`mesocosm/design_docs/2026-10-28_place_graph_engine_plan.md`, SP6).

## Decisions

Answered by Mark on 2026-11-05, in answer order.

- **D1, target (2026-11-05), applies Processes 18:** "consider what the mem
  costs and savings would be for areas 10x, 100x, and 1000x the current areas,
  and if it's worth it then, then do 1". Option 1 was processing first:
  brick-wise growth, with the uniform form cutting memory and clones along the
  way. Measured (Findings, 2026-11-05): the current form passes a gigabyte at
  100x Eponym's area and the collapsed form holds a third of that, so the
  test is met and the plan targets processing first. Offered against
  memory-only collapse, growth only, and dropping it.

  Links: area Stack and engine · draws on processes.18, stack.5 · summary Grow the ground brick-wise and keep uniform bricks as one material; the memory case was tested at 10x, 100x and 1000x area.

- **D2, encoding (2026-11-05), applies Schema 9:** "Compact encoding". A
  uniform brick encodes as its one material, so every ground hash changes. No
  persisted record cites one, and game saves carry no ground (Findings), so
  no save loses its reader. Offered against keeping the bytes identical.

  Links: area Stack and engine · draws on schema.9 · summary Uniform bricks encode compactly; ground hashes change, and no persisted save or record depends on them.

- **D3, Mesocosm's clones (2026-11-05), applies Processes 18:** "Measure
  first (Recommended)". Count scene rebuilds before deciding whether the
  ground is shared rather than cloned. Offered against sharing it now and
  against leaving Mesocosm out.

  Links: area Stack and engine · draws on processes.18 · summary Measure how often Mesocosm clones the ground before choosing to share it.

- **D4, home (2026-11-05), applies Stack 5:** "Ground now, owed to SP6
  (Recommended)". Build it in isometer-core's `Ground`; SP6 gains the
  obligation that nisus keeps the uniform form and brick-wise fill. Offered
  against building in nisus first and against growth only.

  Links: area Stack and engine · draws on stack.5 · summary Build uniform bricks in Ground now; the move to nisus must keep them.

## Constraints

- The 25 call sites reading `brick_materials(key)` then `brick.raw()`
  (isometer-lens, isometer-space, isometer's scene, eponym-client, mesocosm
  runtime and probes) keep compiling unchanged: `raw()` stays `&[u8]` of 512.
- One ground has one encoding. A brick that becomes uniform through edits is
  stored uniform, so a ground edited back to its grown state encodes, and
  hashes, as the fresh grow does. The replay witness rests on this.
- Bricks are still never removed; an all-air brick stays a key (Deferred).
- Source files stay under 600 lines; `ground.rs` is at 588, so new code goes
  in siblings under `ground/`.

## Phases

### UB1. Brick-wise growth

`grow_with` builds each brick's 512 materials in a local buffer and inserts
the brick once, instead of one map lookup per voxel through `place()`.
Cavities and stances keep their per-voxel paths.

- Done when: the new grow equals the per-voxel grow (`PartialEq` and encoded
  bytes) on the Hills, Flat and Rolling fixtures and one with cavities; grow
  time at 1x, 10x and 100x area is measured and recorded in the receipts
  beside 2026-11-05's baseline; isometer-core and isometer tests pass.

### UB2. The uniform form

`Brick` becomes private `Uniform(u8)` or `Dense(Box<[u8; 512]>)`. `raw()`
answers a uniform brick from a static table of 64 filled rows (32 KiB, one per
material up to `MAX_MATERIAL`). Setting a voxel promotes a uniform brick to
dense; at the end of each `write`, `carve` or generation step, a touched dense
brick that has become uniform is demoted. `is_empty` and `get` answer a
uniform brick without a pass.

- Done when: the raw() call sites compile unchanged; isometer, Eponym and
  Mesocosm workspace tests pass (`--all-features --all-targets` where the
  repo's CLAUDE.md asks); a test edits a grown ground and back and finds it
  equal, brick forms included; memory at 1x, 10x and 100x is measured with a
  counting allocator and the clone time at 1x, both recorded in the receipts.

### UB3. Compact encoding

Serde follows the enum: a uniform brick encodes as a tag and one byte. Ground
hashes change once (D2).

- Done when: the edited-back ground from UB2 encodes byte-identically to the
  fresh grow; Eponym's replay and room tests pass with their regrown hashes;
  encoded size and encode-plus-hash time at 1x are recorded; a re-grep finds
  no persisted ground hash (`ground_hash`, "ground hash") in live docs,
  `testing/` or fixtures.

### UB4. Mesocosm's ground clones, measured

Count `SiteScene` builds per second in the running Mesocosm app
(`borrowed_section.rs:211,342`, `app.rs:307` each clone the ground), in a
creative and a survival scene, idle and while the sim runs.

- Done when: the rate is recorded per scene and state. If the ground is
  cloned per frame or per sim step, sharing it (`Rc`) comes back to Mark as a
  question; it is not decided here.

### UB5. SP6 carries the form

- Done when: SP6 in the place-graph plan names the obligation (D4). Written
  2026-11-05.

## Deferred and parked

- **Removing all-air bricks**: a ground could drop a brick carved to air.
  This changes `keys()`, `column_layers` and the dirty queue's meaning for
  projections, so it is its own question.
- **Enclosed-brick culling**: the uniform form makes "solid and buried" a
  cheap check for residency and meshing. Eponym's residency already avoids
  buried bricks by camera footprint; Mesocosm and the board frame whole
  columns (unmeasured).
- **Dense-brick compression** (palettes, run lengths): not before UB2's
  numbers.
- **Paging past 100x**: at 1000x even the collapsed form is about 4 GB, so a
  world that size pages bricks from the seed (`Ground::empty` already names
  paging); the uniform form triples the resident area per byte.

## Findings

- 2026-11-05: game saves carry no ground. `WorldSave` is generator version,
  seed, config, `base_hash` and intents (`eponym/crates/eponym-play/src/world.rs:115`);
  `base_hash` is the genesis digest (`world.rs:287`); the v5 fixture is 827
  bytes. Only the Eponym probe's `Save` holds a `ground_hash`
  (`eponym/crates/eponym-client/src/probe.rs:149-160`), checked against a
  regrown ground on restore, and no live doc or `testing/` file records one.
- 2026-11-05: memory is linear in area. A test terrain 30 to 37 voxels deep
  (65% of bricks uniform) measured 4.2 MB now and 1.5 MB collapsed at 7,478
  bricks; 41.4 and 14.4 MB at 72,819; 408.5 and 135.6 MB at 715,705. Scaled
  to Eponym's world (20,610 bricks, 70% uniform): about 11.7 MB now and 4.0 MB
  collapsed at 1x, 117 and 40 MB at 10x, 1.17 GB and 400 MB at 100x, 11.7 and
  4.0 GB at 1000x (extrapolated).
- 2026-11-05: growth dominates processing. At 7,478 bricks: grow 119-215 ms,
  clone 2.6 ms, encode 6.9 ms (3.87 MB), hash 5.2 ms; grow at 715,705 bricks
  took 23 s. `place()` does a map entry per voxel (`ground.rs:219`).
- 2026-11-05: nisus's `VoxelChunk<T>` is a dense `Vec<T>` with no uniform form
  (`mere/crates/conatus/nisus/src/lib.rs:143`).

## Progress

- 2026-11-05: plan written; D1 to D4 answered; SP6 carries D4.
