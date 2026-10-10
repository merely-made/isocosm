# Isocosm's state witness

**Status, 2026-10-08:** H1 and H2 landed on main and pushed. The plan is
complete.

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

**Where H2 stands, read 2026-10-08.** mere's dynamics grammar plan
(`mere/design_docs/mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`,
§G8 and its 2026-10-06 annotation) lists G8 as open: not briefed, and not
among the lanes F92 set running (G2's rework and G4). Isocosm's needs reach
G8 as four relayed lines; the inventory below is the fuller statement for its
brief.

**mere's answer, 2026-10-08 (ruling 647's request).** Mark ruled on mere's
side, as F116 to F119 of its dynamics grammar plan (relayed by its physics
coordinator; on mere's branch `state-witness`, not yet on main): the crate
is built now, as a third lane ahead of the rest of G8 (F116); it is a leaf at
`crates/system/state-witness`, on serde and postcard only (F117), unpublished
and pinned by rev (F118); and it owns `hash_bytes`, proven equal to
`isometer_core::snapshot::hash_bytes`, *switching `isometer_core` over and
retiring its copy being this repo's change* (F119). Its API forks (ordered or
keyed entries, a label on one side only, the report's shape, a whole-list
digest beside H1's hash, a postcard header) go to Mark there; isocosm's
evidence on each was sent to the coordinator the same day. H2 opens when the
crate reaches mere main, with the F119 switch as its first step.

**Label inventory, from `State` (`shared/isocosm/src/simulation.rs:37`).**
Saves label by field (633); traces label by entity, which per collection is:

| Field | Keyed by | Entity label |
| --- | --- | --- |
| `tick`, `next_action` | scalars | one `clock` entry |
| `population` | cohorts: first `Id` and a `count` | the normalized cohort's first id |
| `sites`, `locations`, `polities` | `Id` | `site:<id>` and so on |
| `lineages`, `events` | `Key` | `lineage:<key>`, `event:<key>` |
| `relations` | the (subject, kind, object) triple | the triple itself |
| `notes` | position in an append-only `Vec` | `note:<index>` |
| `roots`, `released` | `Id` | `root:<id>`, `released:<id>` |
| `record` | hagiograph marks by `Key` | `mark:<key>` |
| `reach` | arrivals by event `Key`, then site `Id` | `arrival:<event>/<site>` |

Two of these are not plain lookups:
- *Population.* A cohort is a range of ids carrying one entity, and grouped
  and individual runs agree only after `Population::normalized`. An entry per
  critter would mean expanding every cohort; an entry per normalized cohort,
  labelled by its first id, keeps the trace as large as the state is.
- *Notes.* A note has no id, but the list is append-only: notes are pushed
  (`simulation.rs:409`, and through the stage, `stage/mod.rs:368`), expiry
  is read against the tick rather than removing anything, a full budget
  refuses rather than evicts (`room`), and the only shrink is the journal's
  rollback to the length an advance began with (`journal.rs:103`). So the
  index is stable for a world's life and labels a note. *Corrected
  2026-10-08:* the first reading said the index shifted on expiry; it does
  not.

## Forks, ruled 2026-10-06

1. Label grain: fields in saves, entities in the traces (ruling 633).
2. The v2 hash: a u64, v1 read through a separate struct (634).
3. Staging: H1 after the repin, H2 when mere's crate lands (635).
4. Encoding: postcard through `snapshot::encode` (636).
5. A v1 save is still checked at every checkpoint, not only its final hash
   (641, 2026-10-07).
6. The probe's trace is draw 0's exact run, in the bench's line format (642,
   2026-10-07).

## Progress

- 2026-10-07, H1: `Simulation::state_hash` is `hash_bytes(encode(..))`, a
  u64, over the same normalized tuple; the SHA-256 survives only as
  `state_hash_v1`. Saves carry `SAVE_VERSION = 2`; `Session::load_json`
  reads the `version` field and sends v1 to `load_v1`, which checks the
  final SHA-256 and, stopping the replay at each recorded checkpoint tick,
  every checkpoint's (641). `Session::advance_traced` and
  `probe::run_exact_traced` feed `--trace <file>` in `isocosm-bench` and
  `isocosm-probe` (draw 0's exact run, 642; refused beside `--crowds`, which
  drops that run). Outside isocosm only `mesocosm-genet` moved: its
  `sim-hash` probe field prints `{:016x}` and its save panel loads through
  `load_json`. Done-conditions: every caller builds on the u64, all eight
  workspaces checking over all targets; the pre-causation fixture loads and
  re-saves as v2; a new v1 fixture with checkpoints at 4, 8 and 12, written
  by the pre-H1 core (`tests/data/v1-checkpointed-world.json`), loads,
  re-saves as v2 and refuses a flipped middle checkpoint, a dropped one and
  a flipped final hash; a v2 save round-trips and refuses a flipped hash;
  a one-unit `PlaceMatter` after tick 5 makes the trace first differ at
  tick 6, the first traced tick after it, and not before. Tests, one
  thread each: isocosm 1,237 (six new in `tests/witness.rs`), the VTT 371,
  Mesocosm 336, isometer 293, isomere 51, wing-integration 4,
  isocosm-overlay 43; Eponym 81 with the two Sortie failures that predate
  the repin. The probe's report for seed 11, one draw, matches the pre-H1
  core's but for timings. Built on one shared target (637): 81 GB for the
  eight workspaces' checks and tests, where the repin's per-workspace
  targets held 126 GB.
- 2026-10-08, H2: isocosm repinned onto mere `329d60d0` (ruling 650; its
  own plan, `design_docs/archive_docs/2026-10-10/2026-10-08_mere_329d60d0_repin_plan.md`), and
  `isometer_core::snapshot::hash_bytes` became a re-export of mere's
  `state_witness::hash_bytes` (mere's F119), held to four values computed
  apart from the crate, before and after. `Simulation::witness` gives one
  entry per field, the world's seed, revision and traits beside the state's
  fourteen (17 in all); `Simulation::entity_witness` one per entity, every
  critter by id with its cohort's digest computed once (649), notes by their
  append-only index. `state_hash` is the field entries' digest (652), and the
  whole-state hash of H1 survives only as `state_hash_v2`. Saves are v3
  (651): each checkpoint and the final state carry their entries, and a load
  names the earliest diverging label ("checkpoint at tick 8 diverges first
  at sites"); v1 and v2 load through the kept readers in `history/read.rs`,
  checked by their own hashes at every checkpoint, and save again as v3. A
  v2 fixture with checkpoints at 4, 8 and 12, written by the pre-H2 core
  (`14d8d9af`), proves the v2 reader. `advance_traced` hands its callback
  the world; `isocosm-bench` and `isocosm-probe` gain `--trace-entries
  <file>`, the crate's framed `Trace` (653), beside `--trace`.
  Done-conditions: a unit placed at one site is named `sites` on load at the
  tick-8 checkpoint and `site:<id>` at tick 6 in the entity trace; both
  execution modes give the same per-critter entries; v1 and v2 refuse a
  flipped middle checkpoint, a dropped one and a flipped final hash; a v3
  save with a changed entry is named, or refused when its digest no longer
  folds. All eight workspaces check; tests, one thread each: isocosm 1,241
  (ten in `tests/witness.rs`), the VTT 371, Mesocosm 336, isometer 294,
  isomere 51, wing-integration 4, isocosm-overlay 43; Eponym 81 with the two
  Sortie failures that predate both repins.
