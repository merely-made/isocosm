# Isocosm Overlay

The overlay contract between a game and the Isocosm sim (wing design record
ruling 154): a game submits **intents** stamped for a tick; the sim returns
**events** (receipts and record entries, each participant's stream derived
from their attention set) and a read-only
**view** of each tick; outcomes a game settles itself come back through the
**handoff** in the sim's own terms and must pass the sim's invariants.

Accepted reading D18 governs this crate: every contract type round-trips
through bytes, which proves no pointer crosses the boundary, and this crate
depends on nothing sim-internal and on no product crate. Every identifier is
an opaque handle minted by the sim, never a pointer into it (wing design
record §5.2 point 6). A game crate depends on this crate and on the sim;
this crate depends on neither.

Version 0.1.0, unpublished; the name is not reserved on crates.io.

Run from the repository root:

```powershell
cargo test --manifest-path shared/isocosm-overlay/Cargo.toml
```

## Layout

- The crate root (`src/tick.rs`, `handle.rs`, `intent.rs`, `attention.rs`,
  `event.rs`, `view.rs`, `handoff.rs`) is the wing-level core: the tick stamp,
  opaque handles, the intent envelope, the attention set, the event record,
  the view handle, and the generic handoff envelope. All game-neutral. An
  envelope names the participant who submitted it and carries either an
  attention change, which every game shares, or the game's own intent.
- `src/mesocosm/` is Mesocosm's vocabulary, the first game's module (Mesocosm
  overlay plan §3; wing design record §5.5): the nudge, which is the one
  directive a player sends (rulings 214 to 216), the player's own acts,
  checkpoint answers and dev intents. A second game's overlay adds a
  sibling module here, not changes to the core.
- `tests/core_roundtrip.rs` and `tests/mesocosm_roundtrip.rs` byte-round-trip
  every type in the crate through `serde_json`, the format `shared/isocosm`
  already saves and replays with (`serde_json::to_vec` / `from_slice`,
  `isocosm::history::Session::save`). `MesocosmHandoff` is the one exception:
  it is uninhabited by ruling (see below), so there is no value to round-trip
  and the test instead checks that nothing decodes into it.

## Forks, ruled 2026-09-25

Building M1 stopped at three forks. Each was shipped at its most easily
replaced option and put to Mark; the wing design record carries his answers.

1. **The tick stamp's type** (ruling 203): a newtype, `struct Tick(pub u64)`,
   over the sim's flat count. Epoch boundaries stay a rule over that count
   (`Session::advance`, `shared/isocosm/src/history.rs`), not a field of it.
2. **Event subscription** (ruling 204): derived from the participant's
   attention set (the record's §5.2 point 4, in the contract's shape since
   ruling 154), typed in `src/attention.rs` under rulings 210 to 213. Nobody
   subscribes separately; the placeholder topic set is gone.
3. **How a place is named** (ruling 205): an opaque handle to a node of the
   place graph (rulings 72, 147), as a nudge's target is. Dev intents keep raw
   coordinates (`WorldPoint`), since they reach the grid directly.

## The attention set

One per participant, whether a player, a servitor or a scenario runner, and
at once the collector's roots, the foreground that runs in detail, and the
source of the participant's event stream (§5.2 point 4). It holds who they
play, what they pin, which may be any pointable thing (ruling 210), the
region their view shows up close (ruling 212), and the game's own care,
derived by the sim from the game's profile, where a group keeps its noted
members exact and runs the rest as a crowd (ruling 211). A game changes it
only through attention changes, which are intents and so logged (ruling
113); the examined region changes only when the up-close view moves to
another, so the camera stays presentation inside a region. Each peer derives
its own participant's stream of what is attended from the shared sim, so the
stream never crosses the network (ruling 213).

Two readings sit under it, not rulings: the envelope names the participant,
since ruling 152 has the sim check that a player directs only who they play
while ruling 153 lets two direct one entity; and the survival filter, only
what the played critter can know (ruling 180), is applied where the stream
is derived.

## Mapping `mesocosm-core`'s sixteen `Intent` variants

`mesocosm-core`'s `Intent` (`src/world/intent.rs:74`) holds sixteen variants.
The overlay plan's §3 splits them under directing (ruling 175); rulings 202
and the readings below settle each one.

| `Intent` variant | Under directing | Maps to |
| --- | --- | --- |
| `Move` | the critter's own act | shaped by nudges, its range grown from them (ruling 216); no contract type |
| `Metabolize` | the critter's own act | the one verb; a nudge to attend to food, or a right click's act on it (ruling 214); no contract type |
| `Consume` | the critter's own act | taking an organ off a carcass; no contract type |
| `Graft` | the critter's own act | incorporation, gated by traits; no contract type |
| `Deposit` | the critter's own act | no contract type |
| `Carve` | the critter's own act | den-making where its home has grown (ruling 216); no contract type |
| `Idle` | loses its role | directing has no per-tick request to do nothing |
| `Speciate` | the player's act (ruling 202) | `MesocosmIntent::Act(PlayerAct { kind: PlayerActKind::Speciate { name }, .. })` |
| `Express` | the critter's own development (ruling 202) | chosen by its methodology, its priorities grown from attention (ruling 216); no contract type |
| `TakeControl` | checkpoint | at a birth, `Birth(BirthAnswer::TakeOffspring)`; at a death, `Death(DeathAnswer { next })` |
| `Resume` | checkpoint | at a birth, `Birth(BirthAnswer::KeepParent)`, the default (ruling 183) |
| `Revise` | checkpoint | `EpochReview(RevisionAnswer)`, the review's committed revision |
| `EndEpoch` | dev | `DevIntent::EndEpoch` |
| `ForceBirth` | dev | `DevIntent::ForceBirth` |
| `Kill` | dev | `DevIntent::Kill` |
| `PlaceMatter` | dev | `DevIntent::PlaceMatter` |

The seven driving intents become the critter's own acts, chosen by its
methodology and shaped by nudges, and only `Idle` loses its role
outright. That split is the overlay plan's reading, confirmed here intent by
intent; it is not a ruling.

**The death checkpoint is a reading too.** `mesocosm-core`'s `TakeControl`
and `Resume` also answered a checkpoint at a loss of control: continue
through a descendant, or let the line go. Rulings 61 and 178 rule the first
half: death is final, the cohort is the pool of further lives, and the
player takes up the next of the lineage. `DeathAnswer` carries that, naming
the next life. No ruling names letting the line go, so there is no such
answer.
