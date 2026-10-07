# Isocosm's state witness

**Status, 2026-10-06:** briefed and its forks ruled (633 to 636); H1 waits on
the repin onto stable (ruling 621), then opens in its own worktree.

Carries out the wing design record's rulings 607 to 610: Isocosm's state hash
joins the family's FNV-1a witness, in balaur's shape (labelled entries and a
first-divergence report); version-1 saves still load (608); the labelled
digest is built once, in mere (609); saves keep labelled checkpoints per
epoch and the tools write a per-tick trace on demand (610). The spec lives
here and the decisions in the record (ruling 616).

## What exists (isometry `cf3b1dbb` onward)

- `Simulation::state_hash` (`shared/isocosm/src/simulation.rs:192`) clones
  the state, normalizes the population (`Population::normalized`,
  `population.rs:113`) and takes SHA-256 over the JSON of (seed, revision,
  world, state) through `crate::digest` (`lib.rs:48`), returning a hex `Key`.
- `State` (`simulation.rs:38`) holds thirteen fields: tick, next action,
  population, sites, lineages, locations, polities, relations, notes, roots,
  released, the hagiograph record, events and reach. Every collection is
  ordered (`BTreeMap`, `BTreeSet`, `Vec`), and no authoritative value is a
  float.
- `Saved` (`history.rs:47`) stores `version`, the genesis and its digest, the
  branch, the entries, the tick, the `state_hash` and a `Checkpoint { tick,
  state_hash }` per epoch boundary (`history.rs:126`). Load replays and
  compares the final hash and every checkpoint, reporting only "checkpoint
  history mismatch".
- `Saved.version` is the crate-wide `VERSION`, which also gates every genesis
  (`genesis.rs:9`) and stamps the bench's reports. Bumping it would refuse
  every existing world, not only old saves.
- The family's witness is `isometer_core::snapshot::hash_bytes`, FNV-1a over
  postcard bytes from `snapshot::encode`, already used by the Mesocosm and
  Eponym trees under `src/legacy/`. Isocosm depends on `isometer-core`.
- `state_hash()` is called across the tree: the history, aggregate, bench,
  probe and scale tools, the tests, and `mesocosm-genet`'s probe state
  (`app/bench/probe_state.rs`, a string field). `tests/shapes.rs` proves a
  v1 save from before a rename still loads, against its SHA-256 hash.
- Frozen receipts under `mesocosm/testing/bench/receipts/` print SHA-256
  hashes; they stay as taken and are not re-run targets.

## Phases

### H1. The witness (no mere crate needed)

`state_hash` becomes `hash_bytes(encode(...))` over the same normalized
tuple, a u64. Saves gain their own `SAVE_VERSION = 2`, leaving `VERSION` and
every genesis alone. A v1 save loads by checking the SHA-256 hash it carries,
the old function kept only as that reader, and saving it again writes v2
(608). The bench and probe tools gain a `--trace <file>` flag writing
`<tick> <hash>` for every advanced tick (610).

*Done when* every `state_hash` caller builds on the u64; the pre-causation
fixture still loads and re-saves as v2; a v2 save round-trips; a planted
one-unit change in a mid-run state makes `--trace` diverge at that tick and
not before; isocosm's suite and every workspace's checks pass.

### H2. Labels (after mere's labelled-digest crate lands under G8)

The state's entries are labelled by stable id and hashed through mere's
crate; saved checkpoints carry their entries per epoch, and a mismatch on load
names the first diverging entry (609, 610). The bench and probe traces carry
entries on request.

*Done when* a planted change in one site's state is named, by label, as the
first divergence on load and in a trace; the v2 format adds the entries
without breaking H1's v2 saves, or the save version moves again with a reader
kept.

## Forks, ruled 2026-10-06

1. Label grain: fields in saves, entities in the traces (ruling 633).
2. The v2 hash: a u64, v1 read through a separate struct (634).
3. Staging: H1 after the repin, H2 when mere's crate lands (635).
4. Encoding: postcard through `snapshot::encode` (636).

## Progress
