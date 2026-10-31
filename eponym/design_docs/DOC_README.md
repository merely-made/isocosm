# design_docs Index

Eponym's documents. The [wing index](../../design_docs/DOC_README.md) links all
three products; the wing's shared documents live in
[Mesocosm's index](../../mesocosm/design_docs/DOC_README.md). Per DOC_POLICY §5
this file is the canonical index for this directory, updated in the same
session as any doc change. Each row is a link, what the doc is for and a dated
status (wing design record, ruling 612); the docs hold the rest.

Names: the products are Isocosm (the VTT), Mesocosm and Eponym (formerly
Paredros), peers; the sim is Cosmograph (cases 1036 and 1037; earlier,
rulings 109 to 111). The native package is `eponym-client`.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- Wing-level architecture lives in the repository root's
  `design_docs/2026-07-30_games_wing_founding.md` and is cited, never
  copied. The three pipeline laws there govern anything crossing between
  games.
- **The invariant is care granularity, not metaphysical person purity**
  (relaxed 2026-07-30; revised 2026-08-13; wing founding record §1). Eponym
  is care for **individuals**: particular others you know. Ordinary play stays
  with one named creature until death. Control may shift through an explicit
  world event or optional player rule, with its consequences recorded. Drift
  means free roster control or care widening to a squad you administer, which
  is Isocosm's granularity.

## Active docs

| Doc | For | Status |
| --- | --- | --- |
| [DOC_POLICY.md](DOC_POLICY.md) | A link to the repository's one documentation policy. | Since 2026-10-06 (ruling 615). |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Eponym's goals and pillars: one named life in a persistent generated world. | Maintainer-owned; revised 2026-08-13. |
| [2026-10-28_eponym_overlay_plan.md](2026-10-28_eponym_overlay_plan.md) | Eponym's overlay (W5): driving a sophont on Cosmograph (E3), the played loop (E4), and the founding charter's open questions. | 2026-10-28: cut; E0 to E2 done; E3 and headed E4 open. |

## Archive

Each archived file carries its own paragraph saying why it moved and what
was carried where. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-10-28/`](archive_docs/2026-10-28/): the overlay plan and the Paredros founding charter, archived whole when the plans were cut into a new set (2026-10-28); their open items, and this index's former open items, live in the new overlay plan.
- [`2026-10-10/2026-09-09_memory_and_remembrance_plan.md`](archive_docs/2026-10-10/2026-09-09_memory_and_remembrance_plan.md): memory and remembrance, F3a and F3b5 landed; folded into the overlay plan's E3 (801).
- [`2026-10-10/2026-09-09_functional_loops_plan.md`](archive_docs/2026-10-10/2026-09-09_functional_loops_plan.md): functional loops and wiring, J and B lanes landed; folded into the overlay plan's E3 (801).
- [`2026-10-10/2026-09-13_genet_document_host_plan.md`](archive_docs/2026-10-10/2026-09-13_genet_document_host_plan.md): the presentation join, P0 to P5 landed 2026-09-14; done (ruling 793), its open items in the overlay plan's §7.
- [`2026-09-26/2026-08-07_paredros_execution_plan.md`](archive_docs/2026-09-26/2026-08-07_paredros_execution_plan.md): ruling 313; F3 to F8 mapped onto the overlay plan's E2.
- [`2026-09-26/2026-09-09_world_conditions_plan.md`](archive_docs/2026-09-26/2026-09-09_world_conditions_plan.md): ruling 315; its schema is the sim plan's §3.1.
- [`2026-09-18/2026-08-10_r4_extraction_review.md`](archive_docs/2026-09-18/2026-08-10_r4_extraction_review.md): retired by W1 (ruling 31).
