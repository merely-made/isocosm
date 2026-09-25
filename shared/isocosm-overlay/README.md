# Isocosm Overlay

The overlay contract between a game and the Isocosm sim (wing design record
ruling 154): a game submits **intents** stamped for a tick; the sim returns
**events** (receipts and record entries, by subscription) and a read-only
**view** of each tick; outcomes a game settles itself come back through the
**handoff** in the sim's own terms and must pass the sim's invariants.

Accepted reading D18 governs this crate: every contract type round-trips
through bytes, which proves no pointer crosses the boundary, and this crate
depends on nothing sim-internal and on no product crate. Every identifier is
an opaque handle minted by the sim, never a pointer into it (wing design
record §5.2 point 6). A game crate depends on this crate and on the sim;
this crate depends on neither.

Local version 0.1.0 follows the 0.0.1 name reservation; this implementation
has not been published.

Run from the repository root:

```powershell
cargo test --manifest-path shared/isocosm-overlay/Cargo.toml
```

## Layout

- The crate root (`src/tick.rs`, `handle.rs`, `intent.rs`, `event.rs`,
  `view.rs`, `handoff.rs`) is the wing-level core: the tick stamp, opaque
  handles, the generic intent envelope, event subscription and event record
  types, the view handle, and the generic handoff envelope. All game-neutral.
- `src/mesocosm/` is Mesocosm's vocabulary, the first game's module (Mesocosm
  overlay plan §3; wing design record §5.5): directives, checkpoint answers
  and dev intents. A second game's overlay adds a sibling module here, not
  changes to the core.
- `tests/core_roundtrip.rs` and `tests/mesocosm_roundtrip.rs` byte-round-trip
  every type in the crate through `serde_json`, the format `shared/isocosm`
  already saves and replays with (`serde_json::to_vec` / `from_slice`,
  `isocosm::history::Session::save`). `MesocosmHandoff` is the one exception:
  it is uninhabited by ruling (see below), so there is no value to round-trip
  and the test instead checks that nothing decodes into it.

## Open forks (Mark to decide)

M1's brief asks for a design choice to stop at and report rather than
settle unilaterally. Three came up while building this crate. Each is
shipped here at the option marked **(shipped)** — the minimal, most easily
replaced representation — so the rest of the contract could be built and
tested; none of the three is meant to read as ruled. Every doc comment on
the affected type points back to this section.

### 1. The tick stamp's type

- **(shipped) A newtype, `struct Tick(pub u64)`.** Same cost as a plain
  alias, but a tick can no longer type-check where some other `u64` (a mass,
  an id) was meant, and it gives the stamp a home for its own methods
  (`next`, shipped). Matches the opaque-handle discipline the rest of the
  contract already uses.
- **A plain alias, `type Tick = u64`.** What the sim itself uses internally
  (`isocosm::schema::Tick`, not depended on here). Zero ceremony, zero
  type safety.
- **A structured stamp,** e.g. `{ epoch: u32, within_epoch: u32 }`. Rejected
  in my own reasoning, not shipped even provisionally: the sim derives epoch
  boundaries from a modulus rule over a flat counter
  (`Session::advance`, `shared/isocosm/src/history.rs`) rather than storing
  them, so this would hand the contract a structure the sim doesn't have and
  would need to keep in sync by hand.

**Recommendation:** the newtype, as shipped.

### 2. The shape of event subscription

- **(shipped) An explicit topic-key set,** `EventSubscription { topics:
  BTreeSet<EventTopic> }`, `EventTopic` an opaque string key. Simple,
  decoupled from anything else, ships today.
- **Derived from the attention set.** Wing design record §5.2 point 4: "one
  attention set... drives the collector, the foreground and the event
  stream," which reads as subscription not being its own mechanism at all,
  just a consequence of what a player's attention set already contains.
  Conceptually the tighter fit, cited by name in the contract's own ruling,
  but the attention set has no type anywhere in the design docs yet, so
  taking this option now means designing that first.
- **A structured filter** (event kind plus an optional entity or region
  scope). A middle ground: more expressive than a flat topic set without
  needing the attention set designed first.

**Recommendation:** the topic-key set for now, so M1 is not blocked on a
type that does not exist yet, with a flag that it likely wants replacing by
the attention-set-derived option once that type exists — the ruling already
points that way.

### 3. How a PLACE is named in a directive

Named in the task brief as the example fork; the options are as given there.

- **(shipped) An opaque place handle,** `PlaceHandle(pub u64)`, naming a node
  of the sim's place graph. Places are already a ruled, first-class sim
  concept (rulings 72, 147; `2026-08-05_place_graph_engine_plan.md`), so this
  keeps the same opaque-handle discipline every other identifier in the
  contract already uses, at the cost of needing the sim to mint and expose a
  handle for anywhere a player might range, avoid or call home.
- **A site reference.** A coarser cell/tile reference, lighter than a full
  place-graph node, if most PLACES targets turn out to be plain cells rather
  than named graph nodes.
- **Raw coordinates,** `[i32; 3]`, matching how `Move`, `Carve` and
  `PlaceMatter` already name a location in `mesocosm-core` today. Simplest,
  but leaks voxel-grid geometry into a wing-level directive and travels less
  well to a game whose world isn't gridded the same way.

**Recommendation:** the opaque place handle, since the place graph already
exists as a ruled sim concept that "where to range, avoid, make home"
describes naturally, and it keeps the contract's discipline uniform. Note
that dev intents are a deliberately different case: `DevIntent::PlaceMatter`
(below) keeps raw coordinates on purpose, matching mesocosm-core's own
dev-tool behavior (`OffGrid` is refused against the grid directly, never a
place-graph node), so this fork is scoped to play-time PLACES only.

## Mapping `mesocosm-core`'s sixteen `Intent` variants

`mesocosm-core`'s `Intent` (`src/world/intent.rs:74`) holds sixteen variants.
The overlay plan §3 splits them three ways, confirmed intent by intent below,
as a reading and not a ruling. Two do not fit the split as given; see the
note under the table.

| `Intent` variant | Split | Maps to |
| --- | --- | --- |
| `Move` | driving | the critter's own act, directed indirectly through `Places` (range) and `Priorities`; no contract type |
| `Metabolize` | driving | the critter's own act, the one verb, directed indirectly through `Priorities` and `Nudge::EatThat`; no contract type |
| `Consume` | driving | the critter's own act, a specialised eating act on a carcass; no contract type |
| `Graft` | driving | the critter's own act, incorporation gated by traits (profile's Mechanics row); no contract type |
| `Deposit` | driving | the critter's own act, plausibly the enactment of `CompetitorStance::Share`; no contract type |
| `Carve` | driving | the critter's own act, den-making toward `Places::home`; no contract type |
| `Idle` | driving | **loses its role**: directing has no per-tick "do nothing" request for a host to submit; a tick where the critter acts on nothing is its own methodology's outcome, not an intent a game sends |
| `Speciate` | checkpoint/review | **does not fit** — see note |
| `TakeControl` | checkpoint/review | `CheckpointAnswerKind::Birth(BirthAnswer::TakeOffspring)` |
| `Resume` | checkpoint/review | `CheckpointAnswerKind::Birth(BirthAnswer::KeepParent)` (the default) |
| `Express` | checkpoint/review | **does not fit** — see note |
| `Revise` | checkpoint/review | `CheckpointAnswerKind::EpochReview(RevisionAnswer)` ("the review's revision") |
| `EndEpoch` | dev | `DevIntent::EndEpoch` |
| `ForceBirth` | dev | `DevIntent::ForceBirth` |
| `Kill` | dev | `DevIntent::Kill` |
| `PlaceMatter` | dev | `DevIntent::PlaceMatter` |

The seven driving intents are marked "no contract type" per the overlay
plan's own wording — they "become the critter's own acts... or lose their
role" — which of the two is this crate's own reading, made here since M1
asks for it confirmed intent by intent; it is not ruled anywhere. Only
`Idle` reads as losing its role outright: the other six describe things a
directed critter still visibly does, just no longer on a host's per-call
request.

**`Speciate` and `Express` do not fit the checkpoint-or-review bucket the
task brief hands down**, which names exactly two checkpoint answers — at a
birth, keep the parent or take the offspring; at the epoch boundary, the
review's revision — and no third slot. Both are definite, player-initiated,
mid-round acts, not answers to either named checkpoint:

- `Speciate { name }` splits the played line and names it, mid-round, on the
  player's own initiative — not a reply to a birth or a boundary.
- `Express { condition }` spends this one body's own reserve to develop a
  discovered candidate now, mid-round — distinct from `Revise`, which is
  heritable, costs the run nothing, and is explicitly boundary-timed.

Both stay unmapped here rather than forced into a shape the task did not
specify or quietly folded into an invented third contract channel. A
plausible next step, not taken here, is a third "review action" channel for
mid-round developmental choices, sitting beside checkpoint answers rather
than inside them — but that is a design choice for whoever confirms this
reading next, not one this crate should assert by shipping a type for it.

`Resume` and `TakeControl` also answer a second, older checkpoint in
`mesocosm-core`'s own doc comments — a control-loss checkpoint ("continue
through a descendant" or "let the line go") — that the ruled Mesocosm
profile does not name. `CheckpointAnswerKind::Birth` covers only the birth
half of that historical pair; if a control-loss checkpoint survives into the
ruled design, it will need a mapping of its own.
