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
  checkpoint answers and dev intents. Each game's overlay is a sibling
  module here, never a change to the core (ruling 197).
- `src/eponym/` is Eponym's vocabulary (Eponym overlay plan §3; the record's
  §5.6 and §9.14), built under ruling 253: the actuation of the one body the
  player drives, asks to peers, tellings with their manner, the player's
  naming and notes, the first-life and death checkpoints, creative mode's
  tag-in, and an inhabited handoff for blows. See "Eponym's module" below.
- `src/vtt/` is the VTT's vocabulary (VTT overlay plan §3; the record's
  §5.7), built under ruling 253: the table's acts per tick, the DM's
  assertions, time at the table, travel, hooks, and the handoff the ruleset's
  resolved actions come back through. See "The VTT's module" below.
- `tests/core_roundtrip.rs`, `tests/mesocosm_roundtrip.rs`,
  `tests/eponym_roundtrip.rs` and `tests/vtt_roundtrip.rs` byte-round-trip
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

## Eponym's module (E1, ruling 253)

Eponym is the game the contract opens actuation to (the record's §5.2 point 2
and §9.14; ruling 60). Its module follows the Eponym overlay plan's §3 as the
rulings of 2026-09-25 and 2026-09-26 settled it: the accepted transition of a
game-side solver crosses per tick, never its input frames (ruling 233); an
ask is a proposal one peer weighs by its opinion of the asker (rulings 60,
63); a telling carries its manner, and posing is that manner, not an intent
of its own (ruling 241); naming and notes are the player's acts (rulings 36,
130, 168); a first life begins when and how the player picks, society the
default (rulings 234, 235); at a death the player becomes a bonded companion
or, with none, another life or a new world (rulings 186, 238); tag-in lives
in creative mode (ruling 187); and the handoff is inhabited, a blow's harm
handed back in the sim's terms (rulings 123, 232). There are no dev intents.

### Mapping `eponym-world`'s intents and `eponym-identity`'s control

`GameIntent` (`eponym-world/src/transitions.rs:59`) holds seventeen
variants, `WorldIntent` (`src/world.rs:33`) two, and `ControlIntent`
(`eponym-identity/src/control.rs:24`) four. The plan's §3 split them under
driving as a reading; this table confirms each, and where it refines the
plan it says so.

| Variant | Under driving | Maps to |
| --- | --- | --- |
| `Generate` | the sim's own | lifting a body from the roster (ruling 75); no contract type |
| `Name` | the player's act | `EponymIntent::Act(PlayerAct { kind: PlayerActKind::Name { .. }, .. })` (rulings 36, 168) |
| `Move` | actuation | `EponymIntent::Drive(Actuation { motion, .. })`, the solver's accepted transition (ruling 233) |
| `Observe` | actuation | a `TimedAct` at a thing or a place; what is seen is the body's own senses (ruling 59) |
| `Take` | actuation | a `TimedAct` at a thing; holding is derived (ruling 53) |
| `Eat` | actuation | a `TimedAct` at a thing; the ledger moves (ruling 38) |
| `Rest` | actuation | a `TimedAct` with no target |
| `Fall` | the sim's own | a consequence of motion, `Motion::fell`; the injury is the sim's (ruling 123) |
| `Wait` | actuation | an `Actuation` with no acts and the body standing where it stood |
| `AdmitAnatomy` | the sim's own | the body's revisions are the record's facts, never a game intent |
| `ReconcileAnatomy` | the sim's own | the same |
| `AttachItem` | actuation | a `TimedAct` at a thing; gear-limited holding (ruling 53) |
| `DetachItem` | actuation | a `TimedAct` at a thing |
| `ResolveVolley` | the handoff | `EponymHandoff::Blow(Blow { harm, .. })`, resolved in the foreground and handed back (ruling 232) |
| `AdvanceMotion` | actuation | `Actuation::motion`: the accepted transition crosses, the input never (ruling 233) |
| `ConfigureMovementProfile` | the game's own | a setting of the game-side solver (ruling 233); no contract type. The plan read it as actuation; the solver's home settles it here |
| `ReviseCanon` | the hagioglyph organ's | no contract type |
| `Carve` (`WorldIntent`) | actuation | a `TimedAct` that edits the ground; the edit is an asserted fact (the record's §1) |
| `InheritSite` (`WorldIntent`) | founding | authored content displacing generated content (ruling 89); the generator's, no contract type |
| `Begin` (`ControlIntent`) | who the participant plays | the attention set's `played`, set by `LifeCheckpoint::First(FirstLife)` (rulings 234, 235) |
| `TagIn` (`ControlIntent`) | creative | `CreativeIntent::TagIn { to }` (ruling 187) |
| `TagOut` (`ControlIntent`) | creative | `CreativeIntent::TagOut` (ruling 187) |
| `Succeed` (`ControlIntent`) | checkpoint | `LifeCheckpoint::Death(Succession { next, .. })` (rulings 186, 238) |

Two readings ride with the module, not rulings: an ask's answer needs no rule
of its own in the sim, since Eponym's three willingness gates
(`eponym-social/src/willing.rs:7-23`) are the act score of ruling 95 asked
of the peer with the asker's standing as a term; and a claim's content
crosses as opaque bytes in the sim's vocabulary, as an event record's payload
does, so a false telling is a version of an event (ruling 117) and never a
type here.

## The VTT's module (V1, ruling 253)

The VTT is the game that fills the handoff (the record's §3.8 and §5.7). Its
module follows the VTT overlay plan's §3 as the rulings of 2026-09-26
settled it: the table's acts cross per tick as one batch, moves among them,
while stances and emotes stay the table's (ruling 244; §5.2 point 1); the
battlemap is projected from the generated volume with the DM's map an edit
over it (ruling 243); the DM's assertions are the world editor's (rulings 89,
156, 190) and a sim-off campaign still writes them as notes (ruling 248);
downtime runs on every player's yes (ruling 246); sharing a character and
the DM playing the unclaimed are on by default (ruling 245); a party travels
the place graph (rulings 72, 205); the sim's arcs reach the DM as hooks
(ruling 191); and a resolved action comes back marked calibrated or not,
Pathfinder 2e calibrated first (rulings 114, 189, 249, 250). No dev intents:
the DM's edit mode is play at the table (ruling 156).

### Mapping `isonetry`'s twenty-three `GameEvent` variants

`GameEvent` (`crates/isonetry/src/protocol.rs:299`) is the replicated unit
every peer applies. The plan's §3 split its variants as a reading; this
table confirms each.

| Variant | Under the contract | Maps to |
| --- | --- | --- |
| `Map(TokenMoved)` | table act | `TableActKind::Move { to }` in a per-tick `TableBatch` (ruling 244) |
| `Map(TilePlaced)`, `Map(ElevationSet)` | assertion | `Assertion::Edit(MapEdit)`, the DM's map an edit over the generated volume (ruling 243) |
| `Map(TokenPlaced)`, `Map(TokenRemoved)`, `Map(TokenFaced)` | the table's | placing and facing tokens is the board's; a character's entry is `Assertion::Character` |
| `TurnAdd`, `TurnRemove`, `TurnAdvance`, `TurnSetOrder` | the table's | turn order is the overlay's timescale (ruling 6); the sim never runs one (the record's §3.8) |
| `Rolled` | the table's | the sim records the band, never the dice (the record's §5.1) |
| `SheetSet` | the table's | the sheet is the ruleset's reading of the one ledger (ruling 38) |
| `Fact(WorldFact)` | assertion | `Assertion::Fact` (rulings 80, 156) |
| `InventorySet` | assertion | possession asserted (rulings 53, 94), as `Assertion::Fact` of kind `inventory` until an item vocabulary is ruled; a reading |
| `ItemTransfer` | handoff | `Transfer` in `Resolved::transferred` when an action moved it; a free gift is an assertion of possession |
| `ItemModifierRevealed` | assertion | `Assertion::Fact` of kind `reveal`, a note revealed (rulings 80, 87) |
| `Generation(GenerationRecord)` | the table's | the host's record of a generator selection; the draw itself is the generator's (ruling 15) |
| `MapStored` | the table's | the DM's prep until activated, then `Assertion::Edit` |
| `MapActivated` | attention | the battlemap the view shows up close: `AttentionChange::Examine(site)` (ruling 212) |
| `World(Faction)`, `World(Place)`, `World(Route)` | founding or assertion | the drawn world's own (rulings 63, 72) or, from the DM's own packs, facts asserted (ruling 89) |
| `World(Character)` | assertion | `Assertion::Character(NewCharacter)` (ruling 36) |
| `World(Law)`, `World(History)` | assertion | `Assertion::Fact`: a world-scope rule (ruling 41) or a history event, a note in the record |
| `World(Storylet)` | assertion | `Assertion::Storylet { asserts, .. }` (the record's §3.9) |
| `World(FactionSheet)`, `World(FactionControlSet)` | the table's | the banked time is the retired faction turn's (ruling 247); which participant plays a faction's people is who they play (ruling 152) |
| `World(PartyMoved)`, `World(PartyPaceSet)` | travel | `VttIntent::Travel` (rulings 72, 205) |
| `World(NodeRevealed)` | knowledge | never sent: arrives as an event from the sim's reach (ruling 84) |
| `ActionResolved` | handoff | `VttHandoff::Resolved` (rulings 114, 154), its rolls and beats left at the table |
| `Emoted` | the table's | a beat, representational by the protocol's own word |
| `StanceSet` | the table's | a declaration the travel resolver reads; a reading |
| `ConditionSet` | handoff | `Condition` in `Resolved`; a DM's ruling outside an action is a `Resolved` with no harm |
| `TransitionResolved`, `TravelResolved` | travel | `VttIntent::Travel`; a crossing between worlds is the sim's own event (ruling 105) |
| `TimeAdvanced` | time | `TimeIntent::Downtime { ticks }`, run when every player's `TimeIntent::Consent` has named it (ruling 246) |
| `CharacterCreated` | assertion | `Assertion::Character(NewCharacter)` (ruling 36) |

## Shapes lifted to the core, 2026-09-26

E1 and V1 first defined three shapes in more than one game module, since a
game's overlay adds a sibling module and changes nothing in the core (ruling
197). At Mark's word the same day they were lifted to the crate root, where
every game reads them: `WorldPoint([i32; 3])` (`src/point.rs`), a raw voxel
coordinate that had appeared in all three modules; `ActKey(String)`
(`src/act_key.rs`), an act named by the ruleset's opaque key, which had
appeared in all three, the VTT's as `ActionKey`; and `Harm`, `Wound`,
`WoundSeverity` and `PartHandle` (`src/harm.rs`), harm in the sim's terms
(ruling 123), which Eponym's and the VTT's handoffs had each carried. The
game modules import them and define nothing of their own.
