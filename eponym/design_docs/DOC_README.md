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
| [2026-09-25_eponym_overlay_plan.md](2026-09-25_eponym_overlay_plan.md) | Eponym's overlay (W5), its executable plan. | 2026-10-17: F3/F4 native level-fixture union and F7's six GPU producer tests plus three ordinary resident-producer tests qualify; final all-target/all-feature compiler check passes. The broad native-level suite retains one earlier severance-camera failure, followed by a passing focused rerun. Three `v1-proof` far-window paging/travel/retarget runtime gates, broader E3 and headed acceptance remain open. One genuine historical archive input is ignored. See the [F receipt manifest](../../../../testing/isometry/f-2026-10-16/README.md). |
| [2026-07-30_paredros_founding_plan.md](2026-07-30_paredros_founding_plan.md) | Eponym's founding record: one embodied life among autonomous named creatures. | 2026-10-10: reference charter; executable work belongs to the overlay plan, including folded functional and memory gates in E3. |

## Open items

- **2026-10-17:** F3/F4's native fixture controls and F7's six GPU tests plus
  three ordinary resident-producer tests qualify by receipt union; the final
  all-target/all-feature compiler check passes. Three `v1-proof` far-window
  runtime gates remain open. F8's six sortie cases and native-edit/world
  replay control, F2/F6 archive and stale-panel retirement, and F5 worn-item
  drop remain verified. Broader E3 and headed acceptance remain open. The wing
  [after-pass](../../mesocosm/design_docs/2026-10-10_after_pass_plan.md)
  owns the triage and exact receipt scope.

## Archive

Each archived file carries its own paragraph saying why it moved and what
was carried where. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.

- [`2026-10-10/2026-09-09_memory_and_remembrance_plan.md`](archive_docs/2026-10-10/2026-09-09_memory_and_remembrance_plan.md): memory and remembrance, F3a and F3b5 landed; folded into the overlay plan's E3 (801).
- [`2026-10-10/2026-09-09_functional_loops_plan.md`](archive_docs/2026-10-10/2026-09-09_functional_loops_plan.md): functional loops and wiring, J and B lanes landed; folded into the overlay plan's E3 (801).
- [`2026-10-10/2026-09-13_genet_document_host_plan.md`](archive_docs/2026-10-10/2026-09-13_genet_document_host_plan.md): the presentation join, P0 to P5 landed 2026-09-14; done (ruling 793), its open items in the overlay plan's §7.
- [`2026-09-26/2026-08-07_paredros_execution_plan.md`](archive_docs/2026-09-26/2026-08-07_paredros_execution_plan.md): ruling 313; F3 to F8 mapped onto the overlay plan's E2.
- [`2026-09-26/2026-09-09_world_conditions_plan.md`](archive_docs/2026-09-26/2026-09-09_world_conditions_plan.md): ruling 315; its schema is the sim plan's §3.1.
- [`2026-09-18/2026-08-10_r4_extraction_review.md`](archive_docs/2026-09-18/2026-08-10_r4_extraction_review.md): retired by W1 (ruling 31).
