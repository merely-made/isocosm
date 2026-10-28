# Receipts: 2026-10-28 sim execution plan

Run evidence for `../../2026-10-28_sim_execution_plan.md` (DOC_POLICY §8).

## E1c, 2026-10-28

Played A1 battery, `played-candidate` arm, seeds 0 to 2, one worker, release,
a quiet machine (`ISOCOSM_REPRO_SEEDS=0-2`, `consumer_reproduction_battery`):

| Tree | Wall time | Lines |
| --- | --- | --- |
| main `3b2db632` | 190.3 s | seed 0 living 13, seed 1 living 243, seed 2 living 44; 3/3 |
| E1c `9664d978` | 171.1 s | identical |

The wall time covers play and the battery's two replays: 540 ticks a seed,
about 0.12 s a tick at main. isocosm's lib tests in debug: 289 passed, the
kept-filing check on; the release suite: 744 passed.

## E7a, E8, D19 to D21, 2026-10-28

Seed 1, scheduling time from `ISOCOSM_PROFILE_ACTIVE` spans (play and two
replays): after E7a 66.7 s, 232 living; periods 10/2/5 10.7 s, 172 living;
30/4/10 2.9 s, 61 living (meals unscaled). Rest's pass was 27.8 s of the
66.7 s. With D19 and D20 as built, seeds 0 to 2 finish in 33.9 s wall
(29.7 s scheduling), living 236, 176 and 75, all qualifying; 190.3 s at
main that morning.

## The head, 2026-10-28

`mesocosm-genet --play --curve 60 --no-trace`, release, at `172d55ed`
(E7, E8, D19 to D21, E4a), unchosen seed 1793177320550931600, population
60, living 67 then 123 at tick 60, `ISOCOSM_PROFILE_ACTIVE=1`: 1,101 s wall.
Span totals (nested, overlapping): `schedule.total` 748 s over 391
advances; `body:rest` 448 s; `foraging.route.read` 446 s (8,642);
`foraging.route.search` 444 s (788); `spatial.prepare` 415 s;
`forecast.source_replay` 333 s (1); `terrain.places` 284 s (356);
`walking.plan` 273 s (25,525); `body:walk:ability:cycle-1` 36 s;
`foraging.sight.read` 19 s (14,703).

Same seed and command, later the same day:

| Tree | Wall | `schedule.total` | `terrain.places` | route search | walking plans | forecast |
| --- | --- | --- | --- | --- | --- | --- |
| `172d55ed` | 1,101 s | 748 s | 284 s (356) | 444 s (788) | 273 s | 333 s |
| cache 1,024 (reverted) | 845 s | 536 s | 117 s (156) | 401 s | 242 s | 204 s |
| E5a-b plus the frame | 613 s | 310 s | 325 s (475) | 7 s (24) | 5 s | 148 s |
| `541d88af` (E5d) | 69 s | 14.5 s | 30.7 s (33, 26 at startup) | 8.3 s (29) | 5.7 s | 0.85 s |

The 1,024-window cache peaked at 6.1 GB. In the frame-only run, 67 of 93
Places misses over 25 ticks came from `check_arrival` on arrivals the game
placed across the whole site. Living at tick 60: 123, 123, 118, 135.
