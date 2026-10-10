# CLAUDE.md, Eponym Repository Role

**Repository location, 2026-09-09:** this product now lives in Isometry's
`eponym/` directory, with its own Cargo workspace. The native package was renamed from
`paredros-room` to `paredros-client` on 2026-09-13, and to `eponym-client` on
2026-09-24.
Read `../CLAUDE.md` for umbrella guidance. The root repository index is
`../design_docs/DOC_README.md`. Shared wing design is now in the sibling
`../mesocosm/design_docs/` inside the same repository. Git operations affect
the whole umbrella; preserve other products' WIP and stage intended paths.

This file defines how Claude Code should behave in this repository. Read it
first when starting any session.

---

## Project Identity

**Eponym** (formerly Paredros; renamed 2026-09-22, wing design record
ruling 109) is a second-person action RPG in a persistent generated world.
You name one creature and ordinarily inhabit that life until it dies. Other
named creatures live independently across settlements, dungeons, ruins,
surface and underground places. They may become allies, enemies, neighbors,
or strangers; none is a unit in a party, and none is required for the player
to build, explore, or live.

Vessel 2 of the Isocosm family, Mesocosm (first person), Eponym (second
person) and the VTT (third person), over one simulator, Isocosm, sharing a
world substrate, a lineage model, and a trust plane. Sharing engine organs is encouraged where the
organ stays verb-neutral (ruled 2026-08-05, wing founding record §1); the
vessels still do not share a genre or their verbs. The wing's
question, ruled 2026-08-07: **continuity under transformation** — here,
whether a community remains itself as control, bodies, and generations
change.

**Early implementation.** The repo holds the name-reservation package, the
design docs, and four crates (`eponym-client`, `eponym-motion`,
`eponym-play`, `eponym-sortie`). Eponym's world is native Isocosm
(`shared/isocosm`); legacy Eponym and legacy Mesocosm are deleted, and only
`isocosm::legacy::campaign` remains, held by the VTT's faction turn until V2
(wing rulings 755, 767, 769 to 771, 2026-10-10). `crates/eponym-client` owns native input, rendering and inspection,
including body sheets, timed actions and the retained S0 room probe
landed 2026-08-08: one room carved into a grown mesocosm hillside, one body
under near-tier kinematics, a fixed input trace with save/reload/replay, and
a headed run presenting netrender's composed master with the room in it,
drawn since 2026-10-09 by Mere's `tenant` over the kiss3d fork (wing
rulings 734 to 736, L7; renderling retired). Its default `r1-proof` profile runs the real room and perspective
camera through the shared brick DDA, now owned by Mere (`modulus`, formerly
`conatus-brick`, pinned by rev); Eponym constructs the shared `BrickMap` from its own
Ground binding. Three further gates landed as opt-in bins: `v1_residency`
(continuous-zoom residency, V1/V1a), `d1_depth` (raymarch depth as the
tenant's depth pre-pass, bodies layered over the traced colour, D1), and `v1b_residency` (the stable capacity-fixed
resident brick cache, V1b), behind the `v1-proof`, `d1-proof`, and
`v1b-proof` features. The S1 willingness owner landed the same
day (now native `isocosm::social`): deeds, standing, confidence, refusal,
standing agreements, and the
premises behind every answer, with the refusal scene as an executable
receipt. S2 (landed 2026-08-08) added the settlement in peer-agency form
to the same module: homes offered with daily work, residence and the
daily round derived from agreement state, so moving out is the agreement
ending. `crates/eponym-sortie` is S3's joint receipt (sim half landed
2026-08-08): the one crate reading both owners, with negotiated
participation, terrain falls as body-revision wounds, the pact-governed
tag-in, the dig rule, and sortie deeds that explain later answers; its
society is now the native one, with S1 and S2's scene kept as its own fixture.
`eponym-play`'s `identity` holds the identity facts both owners
share and neither may own.

The world is native Isocosm, moved 2026-10-10: persistent site meanings,
routes, containment, inherited replacement, multi-author material edits,
generated bodies and items, needs, perception, injury, recovery, death,
deeds, knowing, standing, agreements and homes, and regrow-plus-replay
saves are the sim's, and the sim advances every living member through the
same processes with no observer or selected-subject input. Population,
projects, the autonomous round and the simulation record retired with the
legacy world (770, 771). The game's own half lives in `crates/eponym-play`:
motion, timed actions, the strike resolver, techniques, sheets, admitted
anatomy snapshots, control and the glyph reading's game half. Its
`Movement`, `Bodies`, and `Items` are separate multi-subject systems;
`GameState` coordinates them through one subject-addressed transition grammar
with no control-specific path. Walking and sight go through isometer's
in-site space (`isometer-space`). Navigation remains derived advice.
Traversal and named-life scenarios belong to receipts, not production
vocabulary. F0-F2 are closed; F3 memory,
belief, and standing was active, with F3a (pointable memory and belief)
landed 2026-08-26; E2's families moved onto native Isocosm 2026-10-10,
their done-conditions standing for the after-pass. The executable plan is
`design_docs/2026-09-25_eponym_overlay_plan.md`, phases E0 to E4 (wing design
record ruling 313, 2026-09-26); the 2026-08-07 execution plan is archived at
`design_docs/archive_docs/2026-09-26/`, its S0-S3 and F0-F2 retained as
foundation receipts and its F3-F8 mapped onto E2's families in the overlay
plan's §4.1. The founding plan remains the charter with its phase section
superseded.

`eponym-play`'s `Session` foundation (2026-09-09, then in the legacy world)
composes one `GameState` with historically validated control and
existing-life succession. The player is a participant who takes up its
sophont through native `Take` at each control cut (wing rulings 671, 779):
a sophont is driven, where Mesocosm directs a critter that is not sapient.
Its versioned save supports configurable archive limits. The bounded J1b
`AdvanceMotion` path now owns fractional terrain motion and landing injury in
GameState; precise poses survive replay, while existing navigation/items/combat
retain explicit logical-cell projections. Full contact,
autonomous/social coordination, outsider arrival and host integration remain
open. The current lanes are the overlay plan's E3, §5.1, where the
functional loops plan folded on 2026-10-10 (its text is at
`design_docs/archive_docs/2026-10-10/2026-09-09_functional_loops_plan.md`).

The `crossing` binary (2026-09-05) is the dry damaged-crossing contact
fixture: two restartable body presets, board carrying, tethering, brace,
timed strikes, impairment/recovery, and a readable shared-device HUD.
`eponym-motion::ContactWorld` is a separate fixed-step input/replay probe
using Conatus character movement, not yet a join to `GameState` or F3 evidence.
Its stationary practice body has no decision system. Full encounter and
playtester acceptance remain open in the Eponym overlay plan's E3 and E4.

See `design_docs/PROJECT_DESCRIPTION.md` for the product description,
`design_docs/DOC_README.md` for the doc index, and the wing-level
architecture in the sibling repo at
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`.

## Terminology

- **critter**: the plain organism word, wing-wide.
- **character**: **this game's unit word, ruled 2026-07-31.** A
  faction-association added to a denizen, which is itself a named entity
  (ruling 200) — `character(denizen(critter))`, which this repo's founding plan
  already described as one stable subject with independently versioned
  profile references. Not a coinage: Isometry uses `character` for the same
  artifact, so the two vessels agree rather than each inventing a word. A
  faction is a *relationship*, not a property, which is why the second-person
  vessel is the one that mints characters.
- **denizen**: a named entity. An entity of note, one the simulation
  identifies and remembers individually, becomes a denizen when people
  name it, as they need to refer to it. Naming requires sapience, so
  every sophont is a denizen, but a denizen need not be a sophont (wing
  design record ruling 200, 2026-09-25, amending the 2026-09-20 line).
  The tier is "of note", and notability itself needs no name. This term
  supersedes `borg`; historical rulings retain the old word with
  supersession pointers. See the wing founding record §1 and open
  question 3.
- **sophont**: the term of art for a sapient entity; every sophont is a
  denizen (ruling 200).
- **The battle-frame noun** — the machine a character pilots, if this game
  keeps the Gotcha Force silhouette — **remains unnamed.** It is a separate
  question from the unit word, and not a gap to fill casually.
- **companion / peer**: a relationship to another named creature, not a
  required entourage or roster slot. Never "unit" or "party member" — both
  imply command.
- **succession**: play continuing through another subject after death. An
  existing connected creature and a newly generated outsider are both valid.
- **fili**: lineage across worlds. Not event history.
- **hagiograph**: the history organ, rescoped 2026-09-16 and kept in Mere's
  eidetic core. It judges which events were significant through standing
  marks and feats, gives a generated world its past through **deep time** (a
  run of the world's own simulation before anyone steps in), and later owns
  retelling, remembrance and manifestation, which is this game's lane H.
  Storage of what happened stays in the deed and event journals. Eponym's
  memory/remembrance plan scopes its first consumer; the wing's isoscape
  family plan holds the rulings. Older documents used `tulpa` for the
  memorial sense.
- **tulpa**: Gemot's federated adapter-training lane; not the memorial organ.

## Shared rules

Documents, guidelines, the pipeline laws and the licensing boundary are
the repository's, in the root [`CLAUDE.md`](../CLAUDE.md) under "Rules for all
three products" (wing design record, ruling 615); it loads with this file.

## Important Don'ts

- **Do not add real-time puppeteering of a party.** This is the scope canary,
  narrowed 2026-07-30 when the wing replaced person purity with care
  granularity (wing founding record §1). Eponym is care for **individuals**;
  drift means care widening to a squad you administer, which is Isometry's
  granularity. Permitted: *configure, don't command* (standing behaviour
  agreed in advance, which a peer may refuse). Tag-in may exist as an optional
  player rule or explicit world process, but ordinary play stays with one
  named creature until death. Forbidden: free roster selection and
  moment-to-moment orders to several characters at once.
- **Do not make body switching a menu operation in ordinary play.** Control
  changes through death or a recorded world event. Possession, domination,
  transplantation, cloning, resurrection, and similar exceptions must leave
  material, causal, and social consequences. Record what occurred; do not
  collapse disputed continuity into a universal `same_person` flag.
- **Do not build a Nemesis system.** Procedurally generated rivals with
  promotion hierarchies are patented to August 2036. Generic grudges and
  remembered encounters are fine (Dwarf Fortress prior art); the
  rival-hierarchy-promotion machinery is what to design around.
- **Do not ship the social simulation without its legibility surface.** Depth
  nobody notices reads as procedural noise; the Legends-mode equivalent is
  day-one work, not polish.
- **Do not let relationship drift run without player levers** (the Darkest
  Dungeon 2 lesson): drift the player cannot influence reads as random
  punishment.
- Do not add rollback netcode or a universal CRDT world state speculatively.
  Single-player action comes first, but signed multi-writer world and
  settlement authoring remains allowed. Additive operations preserve
  concurrent claims; each collaborative domain must name its materializer,
  conflict UI, and any true CRDT it actually needs.
