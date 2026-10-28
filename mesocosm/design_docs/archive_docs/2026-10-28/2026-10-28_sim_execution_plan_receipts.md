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
