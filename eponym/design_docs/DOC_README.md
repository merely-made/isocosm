# design_docs Index

Eponym's documents. The [wing index](../../design_docs/DOC_README.md) links all
three products; the wing's shared documents live in
[Mesocosm's index](../../mesocosm/design_docs/DOC_README.md). Per DOC_POLICY §5
this file is the canonical index for this directory, updated in the same
session as any doc change. Each row is a link, what the doc is for and a dated
status (wing design record, ruling 612); the docs hold the rest.

Names: the sim and the family are Isocosm, the second-person game is Eponym
(formerly Paredros), the tabletop is Isocosm: VTT (rulings 109 to 111). The
native package is `eponym-client`.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- Wing-level architecture lives in the Mesocosm repo at
  `mesocosm/design_docs/2026-07-30_games_wing_founding.md` and is cited, never
  copied. The three pipeline laws there govern anything crossing between
  games.
- **The invariant is care granularity, not metaphysical person purity**
  (relaxed 2026-07-30; revised 2026-08-13; wing founding record §1). Eponym
  is care for **individuals**: particular others you know. Ordinary play stays
  with one named creature until death. Control may shift through an explicit
  world event or optional player rule, with its consequences recorded. Drift
  means free roster control or care widening to a squad you administer, which
  is Isometry's granularity.

## Active docs

| Doc | For | Status |
| --- | --- | --- |
| [DOC_POLICY.md](DOC_POLICY.md) | A link to the repository's one documentation policy. | Since 2026-10-06 (ruling 615). |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Eponym's goals and pillars: one named life in a persistent generated world. | Maintainer-owned; revised 2026-08-13. |
| [2026-09-25_eponym_overlay_plan.md](2026-09-25_eponym_overlay_plan.md) | Eponym's overlay (W5), its executable plan. | E0 and E1 done 2026-09-26; E2 to E4 wait on Mesocosm's M3. |
| [2026-07-30_paredros_founding_plan.md](2026-07-30_paredros_founding_plan.md) | Eponym's founding record: one embodied life among autonomous named creatures. | Under rewrite per W1 (ruling 31). |
| [2026-09-09_functional_loops_plan.md](2026-09-09_functional_loops_plan.md) | Functional loops and wiring: session authority, injury and combat, saves. | Rewritten to the record 2026-09-26 (rulings 280, 314). |
| [2026-09-09_memory_and_remembrance_plan.md](2026-09-09_memory_and_remembrance_plan.md) | Memory and remembrance: observer-relative answers, durable history, the hagiograph. | Rewritten to the record 2026-09-26 (ruling 280); F3b5 landed 2026-09-14. |
| [2026-09-13_genet_document_host_plan.md](2026-09-13_genet_document_host_plan.md) | The presentation join: one played session through genet and netrender. | P0 to P4 landed 2026-09-14. |

## Open items

- **`eponym-sortie` fails two tests**, found 2026-10-06 while verifying the
  wing datasheets plan (`../../mesocosm/design_docs/archive_docs/2026-10-06/2026-10-06_wing_datasheets_plan.md`),
  and not that plan's: `a_tag_in_occurs_mid_action_under_the_pact` ("the
  pact never fired") and `an_injury_persists_as_a_body_revision_fact` ("the
  played body was never wounded"). Both fail the same way on the sources
  before its founding change (`d6ffaf20`). The crate last moved the same day
  (`68844aa9`, `f4a40613`). Log: `Code/testing/isometry-datasheets-p3b-eponym.log`.

## Archive

Each archived file carries its own paragraph saying why it moved and what
was carried where. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-09-26/2026-08-07_paredros_execution_plan.md`](archive_docs/2026-09-26/2026-08-07_paredros_execution_plan.md): ruling 313; F3 to F8 mapped onto the overlay plan's E2.
- [`2026-09-26/2026-09-09_world_conditions_plan.md`](archive_docs/2026-09-26/2026-09-09_world_conditions_plan.md): ruling 315; its schema is the sim plan's §3.1.
- [`2026-09-18/2026-08-10_r4_extraction_review.md`](archive_docs/2026-09-18/2026-08-10_r4_extraction_review.md): retired by W1 (ruling 31).
