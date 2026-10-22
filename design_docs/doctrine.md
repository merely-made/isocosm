# The wing's doctrine

What the games wing holds and why, by area: short statements, each citing the
cases it rests on. This is the layer read during work; the cases themselves
(Mark's answers, in his words) are the design record's §0 and each plan's
Decisions section. Rules for keeping it: `DOC_POLICY.md`, "Decisions and
doctrine". Plan: `2026-10-20_decision_links_plan.md`.

**Status, 2026-10-21:** Naming, Licensing, Method, Organs, Presentation, Host, Record and Directing reviewed; the other areas are empty. Each
area's statements are drafted from its cases and put to Mark as a card round
(the plan's L4); until an area is reviewed, its cases in the design record
remain the authority.

## The sim's schema and saves

## Processes and rules

## Ecology

## Bodies and anatomy

## Record and history

### Record 1. Consistency is chosen per domain

Where order matters (the sim's world advance, live tactics, contested cells
and resources), one sequencer accepts intents, each checked against the
revision it was made from; a stale one is refused. The sequencer is a role, a
lease or a turn: it can be handed off, and it is separate from edit authority
(two GMs may both hold authority while one sequences) and from the network
role (host, relay, peer).

Where order does not matter (campaign authoring: facts, drafts, maps,
packs), writers sign concurrent operations, and a type-specific materializer
with the group's policy derives the view, keeping competing work rather than
refusing it.

An offline branch returns as a proposal against the trunk.

Links: draws on 104, 105, 910, 956, 960 · summary Order is chosen per domain: a handed-off sequencer refuses stale intents where order matters; signed concurrent work is materialized where it does not.

*Reviewed 2026-10-22, new. First drafted as "one authority accepts; everything
else proposes against a revision". Mark: "if it composes well with the stack
and doesn't limit the concurrency benefits of p2p networking... client/server,
host/guest are networking and authority axes intersecting. and peers can sit
anywhere on either axis ... there could be two players with GM authority". The
VTT's shared-authority plan (2026-08-08) had already corrected the one-log
view ("too broad"), choosing consistency per domain and making the sequencer a
lease separate from edit mode; the redraft follows it. Mark: "Adopt this
version, in Record and history".*

### Record 2. Three tiers of keeping

The ambient (the soup) is regenerated from basic facts; a thing of note
carries a note, made by the sim as a player would make one; legend never
fades. The record of significant events (unprecedented, legendary or
narratively significant) is never lossy, and the hagiograph judges promotion
by novelty, quality and relevance. Noting and promotion are world events;
collection and merging are work on the record. Collection is garbage
collection rooted in what players care about; examining counts as play; it is
graded, down to just the event.

*Not yet built (sim plan, S4):* graded collection. Today collection folds
only exactly equivalent state.

Links: draws on 4, 69, 70, 71, 80, 125 · summary Ambient, of note, legend; the significant record is never lossy; collection is graded GC rooted in what players care about.

*Reviewed 2026-10-22, new. Mark: "Mark the unbuilt rules as a reading" (marked above), and the gap became an owed item in the sim plan's S4.*

### Record 3. Memory is local and fades

One memory, graded, over one history, from a critter's habit to a sophont's
events; knowledge arrives by an event's reach, a field on the place graph. A
mind keeps what was new to it, mattered to it, was intense or kept happening,
and what it keeps fades at that mind's rate unless renewed; physical notes
(journals, diaries) do not fade. A thing stays of note while some mind
remembers it or some bearer records it, then returns to the ambient. A player
keeps things through who they play and what they pin; Eponym's player notes
are diegetic, the VTT's and Mesocosm's are not.

*Not yet built (sim plan, S4):* fading at the holder's memory rate (a note's
lifetime is today a fixed count set by the process that writes it) and the
four criteria for what a mind keeps.

Links: draws on 5, 55, 127, 129, 130, 131 · summary Memory is local, graded and fades at the holder's rate unless renewed; a thing is of note while remembered or recorded.

*Reviewed 2026-10-22, new, with the unbuilt rules marked at Mark's word.*

### Record 4. Worlds fork and branch

Any state can found a world. A fork founds a new world with its own history,
related by descent and never merging back; a branch is the same world played
apart that returns as a proposal against the trunk, whose clock runs when
played (Record 1). Related worlds share a context without merging (branched
worldlines, celestial neighbours). Where a merge would change lived outcomes
the player chooses: keep the fork, let the replay adjust around them, or let
it win and they become tales. One save holds the sim's history plus an opaque
section per mode, carried unread by builds without that mode.

Links: draws on 7, 104, 105, 126, 274, 444 · summary Forks found new worlds; branches return as proposals; lived outcomes are the player's choice at a merge; one save, a section per mode.

*Reviewed 2026-10-22, new. Mark: "Both as drafted".*

### Record 5. The witness

The state hash is the family's 64-bit FNV-1a witness over labelled per-field
entries of postcard bytes: an equality check for replay, never a content
address. SHA-256 stays for content addresses, draws and anything peers must
trust. Saves keep a labelled checkpoint per epoch; on-demand traces carry one
entry per critter; a mismatch names the first divergence.

Links: draws on 607, 610, 633, 636, 642, 649, 652, 653 · summary The state hash is an FNV-1a equality witness over labelled entries; SHA-256 for trust; epoch checkpoints, per-critter traces.

*Reviewed 2026-10-22, new. Mark: "Both as drafted".*

## World generation

## Organs: glyphs, impresa, hagiograph

### Organs 1. Divinity is earned by the journey

Ascension sacrifices, which means destroys, forms of the world's glyphs. Its
tier (demigod, avatar, avatar-plus, greater divinity) follows how hard the
journey was. A sacrifice's grade is set by how deep the entity's relation to
that magic runs in its story, and scaled by the form sacrificed (an item, then
a technique, skill or ability, then an embodied trait), so a devotee of one
magic who used it greatly can still reach greater divinity. The domain is how
the sacrificed forms were acquired, and a mix of forms mixes it. Collecting
all the magics is favoured, as a world setting. Parked under 279: a design to resume from, not one to build to.

Links: draws on 44, 46, 47, 107 · summary Ascension sacrifices glyph forms; grade is depth of relation scaled by form; the domain is how the forms were acquired.

*Reviewed 2026-10-21, new. Mark on the grading tension between 46 and 107: "Both: depth sets the grade, form scales it"; on the text: "As drafted".*

### Organs 2. Power is the frequency of the domain's process

A domain is named freely, but acts fuel it, and a god whose process rarely
happens is weak, unless its effects are unprecedented or significant.
Choosing a referent is for greater divinities and avatars. Anatomy (vampire,
lich, fey, eldritch) is the other road to long life. Parked under 279: a design to resume from, not one to build to.

Links: draws on 49, 52 · summary A domain's power is how often its process happens; rare but significant effects mitigate; referents are for the greatest.

*Reviewed 2026-10-21, new. Mark: "As drafted".*

### Organs 3. Peeking behind the curtain

Some divine figures read truth beyond their senses and reach: magic that
bends rules 5 (memory is not global) and 59 (senses are the creature's own)
for its bearer. It is a trait a figure may hold, not one its tier grants, and
no play mode or seat. Parked under 279: a design to resume from, not one to build to.

Links: draws on 384 · summary Some divine figures read truth beyond their reach, a magic bending rules 5 and 59; a trait, not a seat.

*Reviewed 2026-10-21, new: kept as drafted when Mark moved divine places (case 48) to World generation.*

### Organs 4. The impresa note

An impresa's note is its record, with a freeform text field (djot) beside
the generated details: a closed subset of enumerated details inside an open
superset of free text.

Links: draws on 82, 85 · summary An impresa's note is its record: enumerated details inside free djot text.

*Reviewed 2026-10-21, new. Put with Organs 5; not disputed.*

### Organs 5. A crate when the abstraction earns it

Crates are made pragmatically, not one per name. An organ is a component in
the sim unless the abstraction earns a crate: it is shared across products or
contexts, or it carries something across a boundary (as `wing-impresa`'s
record type does, and as a `denizen` crate would if it became the way entity
information moves between contexts). Code on a parked rung moves with its
family, unchanged in behaviour, and the rung stays parked.

Links: draws on 201, 673 · summary Crates are made when the abstraction earns one, not per name; parked code moves unchanged.

*Reviewed 2026-10-21, new. Mark, refining the draft "organ words are
components, not published crates": "it's more about being pragmatic and not
inflating the count of crates. not that i don't want any crates. some things
don't need that level of abstraction, you dig?" The principle reaches past the
organs; the Stack and engine round will cite it.*

## Directing and play

### Directing 1. Surfaces derive from knowledge and control

Every surface a participant meets (views, maps, menus, lookups, sounds) is
derived from that participant's knowledge and control: it shows what the
participant knows (the admitted facts and the disclosed map) and offers to act
only on what the participant controls. What a participant may know follows its
seat (player, creative observer, DM), the entity it plays and awareness
granted in play, never its mode; a mode switch changes only how the known is
shown. Settings choose how the known is presented, never what is known. The
sim and the host hold the whole world, so a surface reads through the
participant's knowledge, never from host state directly.

Links: draws on 445, 432, 847, 868, 869, 870 · summary Every surface derives from the participant's knowledge and control; settings choose how the known is shown, never what is known.

*Reviewed 2026-10-22, new. Drafted as a Presentation rule that "the view grants
nothing". Mark: "maybe it's more like presentation is contingent upon
knowledge and control? a rule about how things aren't related is less
interesting than one that describes a relation", asking why a view would be
presumed to grant knowledge (answer: views read from a host that holds the
whole world, the leak cases 868 to 870 guard against); then "Widen it beyond
views". Refined 2026-10-22 in the area's own review with case 445, the
original rule that knowledge follows seat and subject, never mode. Mark: "All
as drafted".*

### Directing 2. Each game foregrounds one rung

Each game foregrounds one rung of the agent ladder (Mesocosm the critter and
its lineage, Eponym the named entity and its factions, the VTT characters
inside polities) and chooses its domain, ensemble, mechanics, controls,
perspective and timescale. The other rungs are simmed and weakly expressed.

Links: draws on 6, 35 · summary Each game foregrounds one rung of the agent ladder; the other rungs are simmed and weakly expressed.

*Reviewed 2026-10-22, new. Mark: "All as drafted".*

### Directing 3. Directing below sapience, driving above it

A non-sapient critter acts by its own methodology and the player shapes it;
a sophont is driven, and directing composes on top as directives through it,
weighed by opinion. One native `Take` takes up either kind of life. Outside
the world a player directs only who they play; two players may direct one
entity, which weighs them by opinion. A creature's senses are its own, but it
suggests what it perceives, so play is not blinded by them.

Links: draws on 59, 60, 152, 153, 671, 679, 779 · summary Non-sapient critters are directed, sophonts driven; a player directs only who they play; co-op direction is weighed by opinion.

*Reviewed 2026-10-22, new. Mark: "All as drafted".*

### Directing 4. The chooser and the bond

A deliberative critter chooses one Choice process a tick, weighing needs,
mood, nudges and its bond; an empty choice may be reconsidered once at a later
pass start. A nudge enters the sim as a logged native command the runtime
translates. Each player's bond is its own weighted relation from a placeless
participant entity to the critter. Standing orders are derived from logged
nudges and their outcomes (desire paths). A suggestion is derived from the
chooser's own record of what it weighed and what the critter knows; nothing
new is kept. An approved journey gives a directed critter a travel goal it
follows when its own choice allows; the bond suffices.

Links: draws on 683, 685, 686, 688, 690, 813, 861, 862 · summary One choice a tick weighing needs, mood, nudges and bond; bonds are weighted relations; suggestions and standing orders are derived.

*Reviewed 2026-10-22, new. 685 named `Receipt.foregone` as a suggestion's
source; the product reads the chooser's live consideration instead
(`readings::suggestions` calls `sim.consider`), so sim-execution D6 cannot
break suggestions. Mark: "Say it neutrally", and the statement names neither
mechanism.*

### Directing 5. Attention

Attention is who the player plays, what they pin (any pointable thing), what
they examine and the game's own care. Examining is what the view shows at full
detail up close: watched, run in detail and logged; far views stay crowd. A
player's event stream carries what is attended. Survival filters through what
the played critter knows and the reach. A location's identity is learned by
observation or telling, its members and extent disclosed separately. The tier
line is directing's.

Links: draws on 210, 212, 213, 691, 742, 876 · summary Attention is play, pins, examining and the game's care; examining is full detail up close; streams carry what is attended.

*Reviewed 2026-10-22, new. Mark: "All as drafted".*

### Directing 6. Death and new lives

Death is final; each game has its trick (a cohort's extra lives in Mesocosm,
non-fungible lives in Eponym, a death the VTT's magic and science can almost
undo). A new life chooses which conditions to carry forward: granular,
randomized or all at once; one not carried keeps its historical event.

Links: draws on 61, 406 · summary Death is final, each game with its trick; a new life chooses which conditions to carry forward.

*Reviewed 2026-10-22, new. Mark: "All as drafted".*

## Presentation

### Presentation 1. One default view

The wing's default view is the 2:1 dimetric at 30°, turned in quarter steps,
in every product, Mesocosm included. It precludes no other view (true and
free-yaw isometric, first person, over the shoulder, Mesocosm's terrarium
section); first person and over the shoulder wait on a perspective camera the
scene does not have yet. A game's grid is a projection: the sim's volume is
cubic, and a tile of any shape is a region of voxels an overlay lays over it.

Links: draws on 18, 382, 387 · summary The default view is the 2:1 dimetric at 30 degrees in quarter turns, in every product; it precludes no other view.

*Reviewed 2026-10-22, new. 382 left open whether Mesocosm opens in the
default; its camera then defaulted to true isometric (35.26°). Mark: "Follow
the wing: 2:1 dimetric, quarter turns", and "Ask the playable-loop lane to do
it": that lane made the change (`c33e69d0`), and it landed on main at
`ace76d61` on 2026-10-22.*

### Presentation 2. The detail ladder

One camera draws five rungs, near to far: live parts over base-grain terrain;
baked sprites over coarse terrain; a mark per individual; aggregate marks for
groups and cohorts; the world map. A thing shows the finest rung its
projected size warrants among those the sim holds, and finer detail arrives
only by examining (212); whether approaching examines is each mode's choice.
Rungs change seamlessly where they share geometry (sprites and live parts,
terrain grains) and by ordered dither with hysteresis where they do not.

Links: draws on 423, 424, 426 · summary One camera, five rungs; a thing shows the finest rung the sim holds for its size; examining lifts detail; rungs change seamlessly or by dither.

*Reviewed 2026-10-22, new, split from the draft at Mark's word. 426 left
open whether examining fires on approach. Mark: "Per mode".*

### Presentation 3. Second cameras

A minimap or overmap is a second camera on the same ladder, held at the world
map's rung. The separate canvas and HUD renderers retire as second cameras
replace them.

Links: draws on 437 · summary A minimap or overmap is a second camera held at the map rung.

*Reviewed 2026-10-22, new: split from the ladder at Mark's word.*

### Presentation 4. Far marks show modifiers

The third and fourth rungs mark what modifies a thing: the world's glyphs of
the effects on it, and for a group each glyph's share of members or the
group's own value, as far as the viewer may know, through the player's filter
with a default per mode. Each glyph's display is rasterised once into a
pixel-icon atlas that marks draw from at integer scale with the scene's depth.

Links: draws on 432, 434, 438 · summary Far marks show the glyphs of what modifies a thing, through the player's filter, from a pixel-icon atlas.

*Reviewed 2026-10-22, new. Mark: "Split P2 in two" (this statement as drafted).*

### Presentation 5. Terrain fits the view

Terrain has a per-axis scale. Its residency follows the view in nested rings:
base grain where the view examines, each ring outward a power of two coarser,
all inside the host's 8 MiB budget. A ring's grain is residency, never the
sim's representation.

Links: draws on 23, 425 · summary Terrain residency is nested rings inside the host budget; grain is residency, never representation.

*Reviewed 2026-10-22, new. Mark dropped the draft's line on worlds past a fixed cap (945, one fixture's decision), which stays a case.*

### Presentation 6. Lighting

The sun comes from the sim's light field (971 to 973). Case 970's fixed sun,
matching the tracer, stays as the bench's default and a host option. Shadows
wait on their testing pass. Case 22 records the lighting the wing aims for.

Links: draws on 970, 22 · summary The sun is the sim's light field; the fixed sun is the bench fallback; shadows wait on testing.

*Reviewed 2026-10-22, new. Mark: "971 amends 970: fixed sun is the bench fallback"; and the draft's wish list dropped, 22 kept as the aim.*

### Presentation 7. Maps and remains

Faction claims and biomes are independent, configurable layers over known
regions. Undisclosed edges clip sharply in every view, and every backdrop,
baked thumbnails included, respects the clip. A resident corpse is seen by
whoever is aware of its place, an in-transit one by no one, and seeing it
gives no command over it. Each follows from Directing 1.

Links: draws on 847, 868, 869, 870, directing.1 · summary Map meanings are independent layers; undisclosed edges clip sharply; remains are seen as their place is known.

*Reviewed 2026-10-22, new: the specifics left here when Mark widened the principle into Directing 1.*

## Stack and engine

## Host and network

### Host 1. One sim host under three product hosts

The embeddable core, `isocosm-host` as a library, founds worlds and holds the
world and its session (Mesocosm's included), the clock with an explicit step
(each product picks wall-clock or explicit pacing), and a driver typed
through a product trait. Three uniform product hosts sit on isomere's
`Assembly` over it.

Links: draws on 884, 892, 896, 898, 901, 925, 926, 927 · summary The embeddable core isocosm-host holds world, session, clock and a typed driver; three product hosts sit on Assembly over it.

*Reviewed 2026-10-22, new. Mark: "All four as drafted"; checked against the code first, each is built.*

### Host 2. Inspection is endpoints

The core's inspection is a Graphshell endpoint behind a feature in
`isocosm-host`; each product's panels are an endpoint module behind a feature
in an existing crate. No new `isometry-` names are made. Every snapshot
records tick, entry count and state hash, and an intent checks it, so a
change through any endpoint stales every endpoint (Record 1).

Links: draws on 896, 900, 901, 903, 910, 920, record.1 · summary Inspection is feature-gated Graphshell endpoints; any change stales every endpoint's snapshot.

*Reviewed 2026-10-22, new. Mark: "All four as drafted"; checked against the code first, each is built.*

### Host 3. Only world keys go through the core

World keys reach the core as a plain `Press`, through a keymap hook on the
product trait; camera, inspection, dev clock and menus stay the window's. A
pick carries its frame's revision, and a stale one is refused (Record 1).
Live input is capped (default 10, configurable); replays and scripts are not.

Links: draws on 954, 955, 956, 957, record.1 · summary Only world keys go through the core; stale picks are refused; live input is capped, replays are not.

*Reviewed 2026-10-22, new. Mark: "All four as drafted"; checked against the code first, each is built.*

### Host 4. A window plays one mode

A window plays one mode and switches in place, keeping the world, the subject
and its embodiment; two modes on one machine are two windows. Each active
window adds to the participant's one attention set; closing it withdraws only
its share.

Links: draws on 443, 815 · summary A window plays one mode and switches in place; windows' attention contributions union into one set.

*Reviewed 2026-10-22, new. Mark: "All four as drafted"; checked against the code first, each is built.*

### Host 5. The family session

`isonetry`, remade, is the family's session layer. One host sequences each
world (Record 1): peers send requests over admitted stream and Graphshell
requests, and the host replicates the accepted native log plus opaque section
changes over Stickleback joined spaces. A replica never reruns a product
adapter to decide an intent. Replicating the full world is a separate grant
from seeing a projection or holding a seat.

Links: draws on 923, 924, 959, 960, 962, record.1 · summary isonetry is the family session layer: one host sequences, replicas take the accepted log and section changes.

*Reviewed 2026-10-22, new. Mark moved 962's rule that products never manufacture keys, grants or trust roots to Stack and engine; the statement keeps 962's separate grant for full-world replication.*

## Mesocosm

## Eponym

## The VTT

## Naming

### Naming 1. The family

The family and its sim are Isocosm; the products are Isocosm: VTT, Isocosm:
Mesocosm and Isocosm: Eponym. Isometry survives only as a plain technical
prefix (the VTT's `isometry-*` crates) and in paths that receipts already cite
(`Code/testing/isometry`, `worktrees/isometry-*`).

Links: draws on 110, 111, 112, 300, 883 · summary Isocosm is the family and the sim; Isometry is only a technical prefix.

*Reviewed 2026-10-21, new. Mark: "Drop the old-name aliases".*

### Naming 2. The of-note ladder

Notability is a ladder of reference. An entity of note is one the sim
identifies; people naming it makes it a denizen; a sapient one is a sophont,
and every sophont is a denizen. A thing of note is a relic, an event of note
a tale, a place of note a location.

Links: draws on 157, 200 · summary Of note is identified, a denizen named, a sophont sapient; relics, tales and locations are things, events and places of note.

*Reviewed 2026-10-21, new. Mark: "As drafted".*

### Naming 3. One word, one sense

Within the wing a word carries one sense. When a word picks up a second
sense, the newer sense takes a new word rather than a qualifier:

- *site* is only the world map's cell; a body's expressed patch is a *tract*,
  where a part joins its *attachment*, an organ's place in a template its
  *situs*;
- *shape* is only a part's form; a process's causal kind is its *causation*;
- *lens* is only a projection; a player's narrowing choice is a *filter*.

Links: draws on 157, 251, 252, 340, 434 · summary A word carries one sense; a second sense takes a new word, not a qualifier.

*Reviewed 2026-10-21, new: proposed from the pattern across its cases, not
Mark's words. Mark: "Adopt it".*

### Naming 4. The rule catalogue's names

A process's causal kind is `Causation` (saved as `shape`, so old worlds
load). A part's seeding is `Grown` or `Acquired`. A process binds a part as
`Binding::Part`, which carries its `shape` and its `functions` from the
world's catalogue.

Links: draws on 340, 341 · governs isocosm::rules::Causation, isocosm::rules::body::Seeding · summary Causation, Grown and Acquired, and Binding::Part are the rule catalogue's names.

*Reviewed 2026-10-21, new. The two cases' readings confirmed: Mark, "Confirm
the readings".*

## Licensing and sources

### Licensing 1. Rulesets

A ruleset ships when its licence lets the wing use it without legal or
community objection. Qualifying: the 5e SRD (CC BY 4.0), Pathfinder 2e (ORC),
Lancer (with attribution), and Ars Magica 5th Edition's open text (CC BY-SA
4.0, with Atlas Games' credit line; anything shipped from it is
share-alike). Daggerheart does not qualify. An open text that informs the
sim's design is a reference: nothing is copied into the sim.

Links: draws on 40, 522 · summary A ruleset ships when its licence draws no objection; an open text informing the sim is a reference, never copied in.

*Reviewed 2026-10-21, new. Mark: "Without the Massif line".*

### Licensing 2. Publication

The wing's crates aim to be published. Licences, metadata and dependencies
are kept publishable, and each release is its own decision. A release in
mere is mere's to decide and is asked there.

Links: draws on 914, 917 · summary The crates aim to be published; each release is its own decision, mere's are mere's.

*Reviewed 2026-10-21, new. Mark: "As drafted".*

### Licensing 3. Licences follow the layer

The VTT's crates, `isocosm-vtt` included, are MIT OR Apache-2.0. Mesocosm,
Eponym, the shared wing crates and their promoted library crates are
MPL-2.0; the only exception route is the fork/vendor criterion in mere's
license posture brief (`mere/design_docs/2026-08-22_license_posture_brief.md`).
Original assets are CC BY-SA 4.0 with an attribution entry; imported assets
keep their own licences and are recorded. Code that moves between layers
takes the destination's licence, relicensed by its author. Each product's
`LICENSES.md` is the grant itself (`../mesocosm/LICENSES.md`,
`../eponym/LICENSES.md`, the 2026-09-03 ruling recorded in Mesocosm's).

Links: draws on 919 · summary Licences follow the layer: the VTT MIT OR Apache-2.0, the games and shared crates MPL-2.0, assets CC BY-SA 4.0.

*Reviewed 2026-10-21, new. Mark, asked whether to cite CLAUDE.md here:
"claude.md should be derived, not authoritative. idk, if there's important
instruction in there it should be a reference to an authoritative record, so
the claude.md points ya to the records instead of bloating claude.md". So
this statement holds the rule, resting on 919 and the products' LICENSES.md,
and CLAUDE.md points here.*

## Method and documents

### Method 1. Evidence is drawn, not picked

A test instance comes from the generator under a seed nobody chose.
Measured evidence is kept apart from lane reports and unknowns; each control
names the fault it would detect, and passing and deliberately failing results
are both kept. A gate is staged with what it gates: viability per family, as
each brings its mechanics (producers, then consumers, then decomposers).

Links: draws on 15, 260, 366 · summary Tests draw under an unchosen seed; evidence is kept apart from reports; controls name their fault; gates are staged with what they gate.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 2. Questions are collected

Forks found in independent work are put together in one round while
authorised work continues; a fork found later joins the next round.

Links: draws on 864 · summary Forks are collected into one round while authorised work continues.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 3. Source size

No source file exceeds 600 lines without a listed technical reason; the
pre-commit hook checks what each commit stages. Oversize and moved files are
split along seams the code already has. Docs are outside the ceiling and are
deduplicated instead.

Links: draws on 193, 592, 593, 594 · summary No source file over 600 lines without a listed reason, hook-checked; splits follow existing seams.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 4. Receipts

Raw receipts live out of tree, under `Code/testing/<repo>/`; the repository
keeps their summaries, sources and scripts. Each lane brief names the receipts
its done-conditions need and caps their total; exploratory runs keep
summaries only.

Links: draws on 255, 953 · summary Raw receipts live out of tree; briefs name and cap the receipts they need.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 5. One home per fact

Each fact has one home; every other mention links to it. Decisions live in
the record and the doctrine distils them; specs live in plans; grants in
`LICENSES.md`; rationale beside the data it explains (datasheet comments);
deferred done-conditions in the one after-pass plan. An index row is a
pointer, a purpose and a dated status, and a doc's own status line is the
current one. Progress entries record landings with their commits and cite
rulings by number; session notes are archived.

Links: draws on 593, 612, 613, 614, 615, 616, 618, 628, 790 · summary Each fact has one home and every other mention links to it.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 6. Dated words

A dated line keeps the words of its date; a later name or ruling reaches it
only as a note. Logs are not rulings: past progress entries are cut to
landings.

Links: draws on 198, 613 · summary A dated line keeps its date's words; logs are not rulings.

*Reviewed 2026-10-21, new. Mark: "Both as drafted".*

### Method 7. CLAUDE.md is derived

CLAUDE.md is derived, not authoritative. It points to the records (doctrine,
record, plans, `LICENSES.md`) and holds only what an agent needs to find them
and to work in this repository; a rule it states lives in a record, and
CLAUDE.md links there.

Links: draws on 615 · summary CLAUDE.md points to the records and holds only what an agent needs to find them and work here.

*Reviewed 2026-10-21, refines 615 (which moved shared rules into the root
CLAUDE.md). Mark, during the Licensing round: "claude.md should be derived,
not authoritative ... it should be a reference to an authoritative record, so
the claude.md points ya to the records instead of bloating claude.md"; then
"Adopt; slim them all now".*
