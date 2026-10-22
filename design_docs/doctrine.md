# The wing's doctrine

What the games wing holds and why, by area: short statements, each citing the
cases it rests on. This is the layer read during work; the cases themselves
(Mark's answers, in his words) are the design record's §0 and each plan's
Decisions section. Rules for keeping it: `DOC_POLICY.md`, "Decisions and
doctrine". Plan: `2026-10-20_decision_links_plan.md`.

**Status, 2026-10-21:** Naming, Licensing, Method, Organs and Presentation reviewed, with Directing 1; the other areas are empty. Each
area's statements are drafted from its cases and put to Mark as a card round
(the plan's L4); until an area is reviewed, its cases in the design record
remain the authority.

## The sim's schema and saves

## Processes and rules

## Ecology

## Bodies and anatomy

## Record and history

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
only on what the participant controls. Settings choose how the known is
presented, never what is known. The sim and the host hold the whole world, so
a surface reads through the participant's knowledge, never from host state
directly.

Links: draws on 432, 847, 868, 869, 870 · summary Every surface derives from the participant's knowledge and control; settings choose how the known is shown, never what is known.

*Reviewed 2026-10-22, new. Drafted as a Presentation rule that "the view grants
nothing". Mark: "maybe it's more like presentation is contingent upon
knowledge and control? a rule about how things aren't related is less
interesting than one that describes a relation", asking why a view would be
presumed to grant knowledge (answer: views read from a host that holds the
whole world, the leak cases 868 to 870 guard against); then "Widen it beyond
views". The rest of the Directing and play area is not yet reviewed.*

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
it": the change is briefed to that lane and owed until it lands.*

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
