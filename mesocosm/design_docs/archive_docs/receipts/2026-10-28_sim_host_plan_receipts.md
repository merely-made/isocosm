# Receipts: 2026-10-28 sim host plan

Run evidence for `../../2026-10-28_sim_host_plan.md` (DOC_POLICY §8).

## H9, 2026-10-31

The world-graph viewer, headed, self-driven by
`cosmograph-host/scenarios/h9_world_graph.scn` (debug build, `--features
viewer`, mere pinned locally at `remote-board-to-stack` `20314ca6`). Seeds read
from the clock; both founded under the bench purpose with 12 sites,
population 200, 6 lineages.

| Run | Seed | Result | Captures |
| --- | --- | --- | --- |
| 1 | 1793457397635410100 | ok, 302 frames | 7, `Code/testing/isocosm/world-graph-viewer/2026-10-31/` |
| 2 | 1793457491267052500 | ok, 301 frames, 7 distinct digests, 0 blank | 7, `Code/testing/isocosm/world-graph-viewer/2026-10-31-run2/` |

Each step asserts the scenario's own readings: tick 0 with regions, sites and
edges drawn; a site opened onto its stored groups; the geography filter
emptying the sites and restoring them; tick 10 at head 10; a scrub to tick 5
with the head still 10 on line 0; a branch at 5 (line 1 of 2) taking a death
(one logged entry) and advancing to 15; the main line back at its head, tick
10, with no entries.
