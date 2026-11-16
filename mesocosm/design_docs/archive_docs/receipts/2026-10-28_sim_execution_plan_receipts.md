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

## The frame tier's two faults, 2026-10-28 (D36 to D38)

`mesocosm-genet --play --seed 1793177320550931600 --curve 20 --no-trace`,
release, with the scratch instrumentation of the handoff (traces under
`testing/isometry/frame-tier-2026-10-28/traces/`).

| Tree | Deaths by tick 20 | Living at 20 | Feeds |
| --- | --- | --- | --- |
| `76f8f5cc`, frame on | 9 (even-id founders 40 to 64) | 94 | 60 |
| `76f8f5cc`, frame off | 2 (52, 60) | 101 | 57 |
| D36, D37, frame on | 2 (44, 58) | 96 | 66 |

At `76f8f5cc` with the frame on, 12 of the 28 consumers left site 5, the
only site holding prey, at tick 2 (4 with the frame off): actor 40's
choices at ticks 2, 4 and 6 were `move:5-4`, `move:4-3`, `move:3-0`, each
the only feasible option, scored 0. At site 3 the far-food table read 56
of prey, 4 foragers and no lineage:3 body there. Frame-on 20 ticks took
61 s after D36 and D37.

The critter (38), `--curve 52`, frame on, at `eaf82b1a` without D38: it
landed at site 3 on tick 45 at `[72, 305, 755]`, admitted with a balance
mass of 5; at tick 46 its mass was 3 and `realized` failed ("posed matter
centre leaves its bearing-foot polygon"); the same at site 0 on tick 50
(admitted at 3, failing at 2). With the frame off (`--curve 90`) its path
diverged at tick 36 and the same failure came at site 1, ticks 70 and 71,
after landing on tick 69. With D38 the critter re-settled at tick 46 to
`[50, 313, 733]` and `realized` held through tick 60.

Headed `look.scn` with the frame off at `76f8f5cc` timed out at 50 minutes
before its third capture. With D36 to D38 and `FRAME_TIER` on, a landings
scenario (`run8-frame-on-landed/look-landings.scn`) shows the critter's
view intact at tick 47 (site 3, after the landing that used to blank) and
tick 59 (after landing at site 4); the critter dies near tick 89 in this
world, so the original scenario's tick-86 capture is mid-journey.

`cargo test -p mesocosm-runtime --tests --no-fail-fast`, debug: 56 passed
and 36 failed at `76f8f5cc`; the same 36 failed with D36 to D38, the only
difference the new re-settle test in `tests/site_habitat.rs`, which
passed (and failed with `settle_drifted` removed). The lib's four failures (`held_advance_tests`,
`verified_question_tests`) are the same with the frame call disabled.
`isocosm` lib: 295 passed, 20 ignored.

## The frame draw, 2026-10-29 (D39, D40)

`physical_swing`'s `unchosen_hungry_starter_can_take_a_supported_food_step`
with its fixture's lineages unpaced (`decides = None`), debug, timed by the
triage of the 36 failing runtime tests: 19 s at `541d88af`, 222 s at
`76f8f5cc`, past 420 s at `b10cebbe` and past 32 minutes at `06b100a2`
(both killed). The branch had the frame on at every one of these points;
`FRAME_TIER` only exists on main.

At `06b100a2`, with wall timers summed per advance (traces:
`Code/testing/isometry/frame-tier-2026-10-28/traces/swing-unpaced-*.log`,
the timers in `instrument.py` and `draw-instrumentation.patch` beside them):

| Tick | Frame draw | Rest of the advance |
| --- | --- | --- |
| 1 | 191.5 s | under 1 s |
| 2 | 425.3 s | about 6 s |

D36 to D38's own work at tick 2: `foraging_wait_score` 432 calls, 252 ms;
`prey_matter` 408 calls, 41 ms; `settle_drifted` 69 ms.

The frame covers the whole of site 0, so tick 1's draw wants all 58 far
members. 23 found a stance (16.8 s); 35 consumers whose two food bodies were
already surrounded each ran the near-food search to its 625-candidate
budget and failed (193.7 s, 5 to 9 s each). Tick 2 tried the same 35 again;
every try failed.

With D39 alone: near-food searches fell to 3 at tick 1 and 1 at tick 2,
but 31 members a tick failed the ordinary search too (81 to 103 s a tick):
the site's ground held 27 to 29 of the 58. The test ran 828 s and failed
its last assertion (the critter still at site 0 after step 8).

With D39 and D40: the test passes in 232 s. Steps took 14, 30, 20, 42, 19,
24, 39 and 27 s; the frame draw 13 to 31 s a tick; posed members rose from
12 at tick 1 to 34 at tick 7.

D40 broke two pins that pass with D39 alone; both re-recorded under D18.
`frame_tier`'s draw test (one advance): under D40 its frame held 0 of 64
after one advance, 1 after two and 2 after eight; D39 alone drew 2 in the
first, the most that quarter's ground holds. It now advances up to eight
times until the frame draws. `physical_swing`'s
`varied_unchosen_starter_walks_on_ordinary_needs_and_keeps_its_feet`: its
critter's first food step is admitted at tick 2, and in that tick it departs
on a journey (with D39 alone it stayed; 201 s); its pose is now asserted
realized exactly while it is resident. A cap of four failed tries an advance
was measured as an alternative: the frame test passed, the varied test failed
the same way, and the unpaced test failed its last check in 342 s.

`isocosm` lib: 295 passed, 20 ignored; `frame_tier`: passed.
`cargo test -p mesocosm-runtime --tests --no-fail-fast`, debug: 58 passed,
36 failed, the same 36 as at `76f8f5cc` and with D36 to D38; the two
passing beyond that are D38's re-settle test and D39's
`a_drawn_member_whose_food_is_crowded_takes_the_ordinary_search` (traces:
`runtime-tests-d39-d40.log`, `isocosm-lib-d39-d40.log`).

## physical_swing unpaced, 2026-10-29

Paced (D19), `unchosen_hungry_starter_can_take_a_supported_food_step`'s
critter (entity 6, `lineage:1`, period 2) decides on even ticks: tick 1
admits no step, tick 2 admits its food step, and it is still resident at
site 0 at tick 12, so the test's departure check fails too (12 ticks, 265 s,
debug). Its fixture unpaced, as Mark's group-1 answer to the runtime triage
asks ("Fix replay, unpace where allowed"): the file's three tests pass in
255 s (`cargo test -p mesocosm-runtime --test physical_swing`, debug).

## physical_swing after case 1057, 2026-11-01 (D47)

`unchosen_hungry_starter_can_take_a_supported_food_step`, unpaced, debug,
failed its departure check after `df93c8ba` merged case 1057: the critter
was still at site 0 after eight ticks. Bisected: it fails at `ca54c521`
(1057) and passes at its parent `75612a3c`, whose critter departs at tick 8
for site 3 (`move:0-3`). The critter's choices, printed per consideration:
at `75612a3c` its foraging wait fell to 0 in tick 8's last consideration
and the moves `move:0-1`, `move:0-2`, `move:0-3` tied at 0; at `df93c8ba`
the wait stayed 1100 through tick 8 and it chose `ecology:feed-1` at tick
9. It was still resident at tick 14, where a birth question holds the run.
The members travelling matched tick for tick through tick 8 (2, 7, 7, 7,
10, 10, 40). Traces: `Code/testing/isocosm/frame-tier-2026-10-28/traces/`
`swing-1057-*.log`, `swing-choice-*.log`.

A nudge toward site 1 after the food step at the default sway (1000, a
fresh bond of 250, so 250): still resident at tick 8. At sway 5000: departs
at tick 2 by site 1's border (`side: 1`), the old pose withdrawn; 54 s.
With D47 built, `physical_swing`'s three tests pass in 65 s.

## EX4 on the bar's founding, 2026-11-08 (case 1174)

Fixture: `shared/cosmograph/tests/world_time_year.rs`,
`a_bodied_world_time_web_holds_two_years`, release, debug info off, run as
`WORLD_TIME_MONTHS=2 WORLD_TIME_REPORT=<abs path> <test exe> --ignored --exact
a_bodied_world_time_web_holds_two_years`, with `WORLD_TIME_EXECUTION=individuals`
for the other mode. Seed 9, 240 bodied founders, ten lineages, a laid 2x2 map,
world-time rates, experimental food contract; 87,660 minute ticks; living 240,
273, 311 at the three samples. Every run below ends at state hash
`1988914608126411875` with the same history digest and monthly samples.
Wall-clock on the shared Windows laptop while another lane worked.

| Build | Grouped (s) | Individuals (s) |
| --- | --- | --- |
| base `11bbea8c` (plus spans only) | 230.4 | 275.7 |
| A: re-file only on a condition a gate reads | 74.9, 105.4 | 76.3 |
| B: no per-act copy of the definition | 68.7, 70.9, 60.7 | 76.9 |
| C: a missed risk-only draw stops at the draw | 36.2, 36.4, 37.1 | 36.4 |
| E: agentless passes visit only world bodies | 35.1, 34.3 | 33.9 |
| D: a missed scheduled draw is not staged | 28.6, 28.7 | 28.1 |
| final (debug check fixed, spans trimmed) | 28.6 | 28.1 |

Base profile (`COSMOGRAPH_PROFILE_ACTIVE=sum`, 268 s with spans): act commits
68 s over 438k accepted acts, 63 s of it in `world:weather`'s 116,880 commits
(four world bodies every three ticks, each re-filing its site's residents);
21.2M act evaluations, 21.0M of them hazard misses (`ecology:hazard-5` 5.8M,
`-0` 5.6M, `-7` 2.6M), at about 1.3 us head, 2.3 us requirements and 0.6 us
draw each; feeding passes 13 s, half of it their plans. After D (33 s with
spans): hazard passes 11.7 s, shared-ground passes with their plans 10 s,
`ecology:sunlight` 2.7 s, `world:weather` 0.8 s.

Debug suite at the final tree: `cargo test --tests --no-fail-fast` in
`shared/cosmograph`, 77 binaries, 888 passed, 37 ignored, none failed. The
first lib run failed 34 tests on `passed_by`'s debug check, which read an
agentless pass going past non-world groups as a filing fault; the check now
allows it.

Two years at the final tree, Grouped, seed 9 (`WORLD_TIME_MONTHS=24`): 953 s,
hash `8216130930403601927`. Seconds per month and living at its end: 13.5
(273), 15.0 (311), 16.6 (341), 19.2 (393), 21.7 (426), 24.6 (425), 25.5 (441),
29.0 (500), 31.3 (518), 34.3 (545), 39.5 (598), 44.3 (612), 45.9 (657), 49.3
(731), 54.6 (727), 58.8 (787), 69.1 (765), 61.7 (626), 54.7 (575), 52.8 (534),
46.9 (504), 47.0 (540), 50.5 (497), 47.6 (492). The receipt's trajectory
reads Passed. This was a timing run, not the bar's measured run, whose horizon
and seeds case 1174 leaves to be asked.

## E7b, death drawn once, 2026-11-08 (case 1181)

Branch `lane/e7b` from `636c72d2`. Distribution:
`cargo test --release --test death_drawn -- --ignored --exact
a_death_drawn_once_matches_the_per_tick_draw_in_distribution`, 20,000
replicates per world-time role at a year of 365 ticks, 1,999 permutations,
seed 1181, bound 50 per mille, alpha 0.05, Holm across the five readings; 29 s.

| Comparison | Distances | Mean death ages (per tick / other arm) | Pass |
| --- | --- | --- | --- |
| per tick against drawn once | 0.0066, 0.0060, 0.0080, 0.0066, 0.0066 | 1677.8/1680.4, 1096.6/1092.3, 1490.9/1483.6, 1853.3/1853.3, 524.3/527.1 | yes |
| per tick against per tick (positive control) | 0.0055, 0.0070, 0.0093, 0.0066, 0.0055 | 1677.8/1672.0, 1096.6/1093.7, 1490.9/1481.8, 1853.3/1850.7, 524.3/527.2 | yes |
| per tick against doubled odds (broken draw) | 0.2450, 0.2493, 0.2544, 0.2454, 0.2500 | 1677.8/1122.2, 1096.6/745.2, 1490.9/1009.4, 1853.3/1333.9, 524.3/318.2 | no |

Roles in order: producer, grazer, omnivore, predator, decomposer. Drawn once:
every Holm-adjusted equivalence p at or below 1.01e-7, every difference p 1.0.
Broken: every difference p 0.0025, every equivalence p 1.0.

Fixture (`WORLD_TIME_MONTHS=2`, seed 9, release): Grouped 16.7 and 16.8 s,
Individuals 16.8 s, hash `5242331604158281548`, history digest and samples
alike across the three runs; living 240, 267, 298. Per-tick draw at
`636c72d2`: 28.6 s Grouped, 28.1 s Individuals. Summed profile, 19.6 s with
spans: hazard passes 0.66 s (11.7 s of 32.7 s before), `ecology:sunlight`
2.2 s, `ecology:feed-1` with its plan 2.8 s.

World test (`tests/death_drawn.rs`, debug): the bar's founding at 120
founders with every hazard's year set to a day; 96 hazard deaths in two days,
each on its drawn tick; matter constant; Individuals and Grouped alike; a
save loaded in either mode replays and goes on alike.

## 2026-11-12 played tick profile

Base `3e453728`, with the fixed-point sunlight and curve profiling changes.
Windows release `mesocosm-genet`, `COSMOGRAPH_PROFILE_ACTIVE=sum`,
`COSMOGRAPH_PROFILE_LIMIT=0`, `--play --curve 20 --seed 18181373014846284518`.
Startup and the curve's observation sampling are outside the step clock.
The host reported 67 living at tick 1 and 151 at tick 20. This run shared
the machine with builds and tests.

| Measurement | First run (s) | Repeat (s) | Corrected sunlight (s) | Calls per run |
| --- | ---: | ---: | ---: | ---: |
| Played steps | 157.82 | 108.75 | 125.74 | 20 |
| Food route search | 142.39 | 100.22 | 112.45 | 33 |
| Walking plan | 131.11 | 92.52 | 103.29 | 2,315 |
| Smell read | 5.45 | 3.09 | 4.31 | 605 |

Spans nest, so their totals overlap. This is a profile of the current
workload and a lead for locating cost, with a different live population
from the older E10 timing. It does not establish a timing regression. The
third run includes the corrected fixed-point cosine coefficient.

## 2026-11-12 sunlight replay

`cargo test --manifest-path shared/cosmograph/Cargo.toml --test light
--test ecology_capture_890 --test epoch --test witness` passed 34 tests.
The `sunlight_platform_check` example, built for native Windows and
`wasm32-unknown-unknown`, matched `e2a1a91c24a0622f`. WebAssembly ran in a
Node worker. The digest covers 24 seasonal dawn and dusk cases, with each
case resumed in Grouped and compared with uninterrupted Individuals.

## 2026-11-16 E11 H0, where hashing time goes

A lane measured the simulator as it stands at `05e8adf1`, Windows release,
on a shared machine (spreads are wide; one configuration was rerun and its
per-lineage document sizes matched exactly). Temporary counters and timers
around `witness`, `entity_witness` and `Session::save`, with positive
controls firing in the same runs. The harness, the instrumentation patch and
the raw outputs are kept outside the repository in
`Code/scratch/nested-voxel-measure/h0-harness`; nothing was committed.

Simulator level, `Session::advance(1)`, 30 ticks after 10 of warm-up
(median). A1 founds 256 on a 3x2 grid and grows to 543 bodies by tick 40;
world-time founds 240 on 2x2. Ten times is population x10, on the same grid
or on about ten times the sites. No hash runs inside `advance`.

| World | Tick (ms) | `state_hash` (ms) | Population preimage (bytes) |
| --- | ---: | ---: | ---: |
| A1, 256, Grouped | 20.1 | 6.9 | 664,677 (543 bodies, 372 cohorts) |
| A1, 256, Individuals | 10.6 | 2.9 | same |
| A1, 2560, 6 sites | 325 | 119.6 | 12,247,576 (7,780 bodies) |
| A1, 2560, 54 sites | 716 | 130.6 | 11,010,582 (7,900 bodies) |
| world-time, 240 | 0.32 | 0.92 | 350,810 (244 bodies) |
| world-time, 2400, 4 sites | 4.7 | 19.0 | 3,392,017 |
| world-time, 2400, 36 sites | 7.9 | 19.8 | 3,377,117 |

A real body's document is 0.14 to 5.0 KB by lineage, about 1.2 KB a body in
A1 and 1.44 KB in world-time. The hash runs at 0.2 to 0.43 GB/s.
`witnessed()` clones the whole state and normalizes the population on every
call: about a quarter of A1's hash time, 6 percent of world-time's.

Played, through `Runtime::bench_placed` and `step(1)`, A1 generated,
Grouped. The hash is taken only in rounds where the played critter's frame
window moved (`frame_attention`, `commit`, `World::cut`).

| Population | Step median (ms) | Witness calls | Share of step time |
| --- | ---: | --- | --- |
| 256, seed 1, 30 steps | 477 | 7, 17.1 ms in all | under 0.12 percent |
| 256, seed 3, 30 steps | 467 | 2, 4.7 ms | about 0.03 percent |
| 2560, seed 1, 20 steps | 3,341 | 1, 20.7 ms | about 0.03 percent |

Reads and saves through the runtime, at 256 and at 2560: `loop_reading`
12 to 13.6 ms and 54 ms; `in_site` 0.004 ms and 30.4 ms; `save_world` 6.5 ms
(55 to 63 percent witness) and 38 ms (77 percent); `resume_world` 16 to 30 s
and 90 s, of which the witness is 4 ms and 32 ms, the rest replay from
genesis. Where a test or bench hashes every tick, the hash is 28 to 37
percent of an A1 tick and 240 to 400 percent of a world-time tick.

Not measured: the genet window's scene refreshes (about two hashes each,
30 ms apiece at 2560), Eponym, and the family host's commit gate (about four
saves a commit, by reading). Seeds 2, 4 and 5 stalled at a death checkpoint
in the harness.
