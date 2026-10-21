# The wing's doctrine

What the games wing holds and why, by area: short statements, each citing the
cases it rests on. This is the layer read during work; the cases themselves
(Mark's answers, in his words) are the design record's §0 and each plan's
Decisions section. Rules for keeping it: `DOC_POLICY.md`, "Decisions and
doctrine". Plan: `2026-10-20_decision_links_plan.md`.

**Status, 2026-10-21:** Naming reviewed; the other areas are empty. Each
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

## Method and documents
