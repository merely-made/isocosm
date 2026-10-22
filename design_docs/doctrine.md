# The wing's doctrine

What the games wing holds and why, by area: short statements, each citing the
cases it rests on. This is the layer read during work; the cases themselves
(Mark's answers, in his words) are the design record's §0 and each plan's
Decisions section. Rules for keeping it: `DOC_POLICY.md`, "Decisions and
doctrine". Plan: `2026-10-20_decision_links_plan.md`.

**Status, 2026-10-21:** Naming and Licensing reviewed; the other areas are empty. Each
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

## Directing and play

## Presentation

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
