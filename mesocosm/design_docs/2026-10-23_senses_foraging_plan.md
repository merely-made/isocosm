# Senses and food-directed walking

**Status:** implemented and qualified, 2026-10-23.

Mark requests ordinary foraging that seeks food an organism can actually detect,
using existing perception and cached terrain, followed by an unchosen moving
capture. This slice joins native choices to geometric sight; richer sensory
modalities, learned recognition and remembered food remain open.

## Phases and done-conditions

1. Read living sensory parts through the existing native function/system and
   placed part graph. Reuse Places sight and the shared terrain cache. New
   foraging rules opt in explicitly; recorded older rules replay unchanged.
   Food within feeding reach remains available by contact. Controls must reject
   hidden, out-of-range and sensorless remote meals and admit revealed food.
2. Connect detection to the existing food candidate/choice and route admission
   paths. Exercise terrain edits, actual identity/visit, rotated anatomy and
   overlapping local windows. Reuse the same terrain cache for authored picture
   sight without treating local-window patch identities as universal.
3. Build the current graph and capture the first eligible clock-drawn world
   walking with intermediate GPU frames and zero developer world intents. Record
   detection evidence for accepted food steps, replay and bounded timing costs.
4. Reuse native whole-body route, foot swing, support and occupancy admission
   while selecting food. A blocked visible meal must not prevent choosing a
   reachable alternative. Bound total search work, share it with the route
   provider, and retain recorded worlds' actual rule semantics. Controls cover
   blocked alternatives, no route, terrain and occupancy changes, and replay.
5. Run a fresh first eligible unchosen seed and capture actual intermediate
   moving frames. Record chosen food and complete supported route witnesses,
   failed searches, timing and source/lock provenance. Recheck older histories.

Receipts: `Code/testing/mesocosm/senses-foraging-2026-10-23/`, total retained
cap 100 MB. Reuse the existing playable target and named foraging Cargo home.
The reachability follow-up uses `Code/testing/mesocosm/reachable-foraging-2026-10-23/`
with its own 100 MB retained cap; existing receipts are preserved.

## Findings

2026-10-23: native `spatial/foraging/food.rs::food_candidates` checks appetite,
diet and actual occupied-box distance but not detection. Shared
`isometer-space::Places::sees` already casts terrain rays. Native
`systems::routes` and living `function:sense` allocations identify realized
senses; `embodiment` resolves the admitted anatomy and yaw. Runtime authored
picture sight currently derives fresh dense geometry and assumes other local
windows share its patch identities. `Simulation::site_geometry` already caches
exact terrain and Places across immutable simulation clones.

Implementation choice: existing `Foraging::search` bounds sight, living routed
sensory tissue supplies eye cells, and any occupied part of food may be seen.
This does not yet model light sensitivity, field of view, smell or sensory
acuity by tissue area. It changes native eligibility without adding knowledge
facts or filtering the creative overview.

2026-10-23 follow-up: `food.rs::foraging_target` selects the nearest detected
meal before `route.rs::geometry` searches for contact. Failed searches therefore
do not inform the next choice. The route search shares native Grounding and
Places, but checks the complete swing only on its first edge. Founding's
`Grounding::near_food` guarantees a supported stance within search distance,
not a connected walk to that meal. Reachability must remain a derivative of
the actual body, terrain and occupancy; it must not become saved authority.

The retained world's first critter 6 has four supported adjacent stands but
every first swing fails: two lose the bearing-foot polygon and two intersect
terrain at the first intermediate sample. Increasing the route-node budget
cannot repair those first edges. Critters 19, 27 and 33 have actual admitted
first swings; this does not establish that their entire old planned paths
were traversable. The exact pending-cut probe is retained with its old binary
hash and source against `dcb07005` in the follow-up receipts.

## Progress

2026-10-23: traced the native choice, native part/system, geometric sight and
terrain cache paths; implementation started.

2026-10-23: reachable-food follow-up started. Inspect the retained failed world,
then consolidate food selection and native route planning before the fresh
unchosen acceptance run. No support, climb or collision limits are relaxed.

2026-10-23: the follow-up passes 48 optimized controls: 26 runtime units, ten
foraging/placement cases, seven sensory cases and five route cases. Native
food choice and the provider now share one bounded multi-goal search. It reuses
Grounding, Places and the existing full swing/support checks. Candidate paths
validate each edge once; failed edges are excluded before trying alternatives,
with one shared node budget across meals and retries. Only the first adjacent
step is published. Biology and detection are read again each round, and exact
physical-input caching shares successes and failures across immutable clones.
New physical rules opt in with `Foraging::reachable`; the omitted field keeps
real older histories' target choices. The controls exercise an inaccessible
nearer meal with a reachable alternative, edited forks, changed resident
geometry over identical terrain, actual cohort identities, full route contact
and omitted-field semantics. The playable build and unchosen gate are next.

2026-10-23: the follow-up's unchosen run starts from clock draw
`1792785704322733000`. The existing structural ecology/placement checks accept
candidate 101, seed `12722265660016199268`; walking outcomes are not a startup
selection criterion. No startup code changed. Three ordinary ticks accept all
16 requested food steps across seven critters, including the default followed
critter 6. The native hash is `27b89789294739d2`, with three births and no deaths.
`inspect_foraging` reconstructs each pending cut, checks its sensory ray, checks
every future native neighbour and full swing, and confirms contact with that
exact food. All 16 witnesses pass. Critter 8's first route to food 57 excludes
four invalid swing edges before finding a complete four-step detour.

The original capture itself contains 24 actual GPU frames, 20 with supported
swing, zero projection failures and zero visible native box intersections.
There is no camera rerun or developer world intent. The first pending-step
census at site 3 finds seven complete routes among eleven nonempty physical
searches. Four searches find no route in their local component, with only
1, 2, 4 or 15 nodes; none exhausts the node budget. Two report intermediate
swing/terrain intersections. These are withheld walking choices, not proof of
global unreachability or evidence that all bodies can walk on these slopes.

The capture logs all 5,936 native spans below the 65,536 cap. There are 337 route
reads and 54 physical searches; the searches total 2.407 s (median 43.37 ms,
maximum 201 ms). Route reads include empty candidate sets and cached results,
so their difference is not a cache-hit count. Cold Places still dominate at
25.83 s across 17 builds. The 107.69 s process time includes all 101 structural
candidates and three live ticks. These are this run's CPU wall spans, with
nested spans overlapping, not a cross-seed speedup or a frame-rate claim.

All five retained older native histories still replay to their recorded
hashes, including the immediately previous sensing world. Receipts and the
lossless moving preview are in the named follow-up folder under its 100 MB
cap, with source, exact lock and optimized binary provenance.

2026-10-23: final receiving checks pass 56 cases: 26 runtime units, ten
foraging/placement cases, seven sensory cases, six route cases, three generated
template cases and four whole-site cases. Two existing profiling cases remain
ignored and the multi-seed walk search remains filtered in favour of the
required unchosen capture. The extra route control warms a shared cache, then
changes an inedible resident's geometry while leaving actor, meal, sight and
terrain unchanged; the edited clone loses its route and the original retains
its original witness. Production Rust sources remain unchanged from the
qualified capture; only that additional integration control was added after it.

2026-10-23: the first navigation controls exposed sensorless authored fixtures.
They now allocate one existing body cell to sensing; blindness has a separate
negative control. Ordinary generated ecology also leaves both its world and
carried organ-system maps empty, even when a recipe has sensory cells. New
physical founding now declares the existing default nervous network where no
sensory source network was declared, carries realized sensory networks on
founders, and retains them in developments for growth and offspring. This is
limited to sensory networks and new physical founding; older saved worlds are
not regenerated. Outdoor sight candidates now include the air above patch
surfaces, leaving the actual endpoint range to reject distant raised parts.

2026-10-23: phases 1 and 2 pass 50 optimized controls: 26 runtime units, 10
ordinary foraging/placement controls, seven sensory controls, three generated
template controls and four whole-site controls. Two existing runtime profiling
tests are ignored and the older multi-seed ordinary-play search is excluded in
favour of the required first clock-drawn headed run. The sensory cases include
terrain hiding and revealing food, loss of sight without losing cached terrain,
blind contact feeding, shorter range, anatomical yaw, actual cohort identities,
window boundaries, stale picture queries and omitted-field legacy semantics.

The first headed build exposed an upstream Cambium atom `AsRef` ambiguity when
resolving fresh Genet `14f7075a` and Netrender `8a3d289a`. The pinned Mere
`a6d44855` records Genet `7d1d89db` and Netrender `ec15eb26` in its own
lockfile. Qualification uses that recorded pair, the current Mere/Eidetic/Gemot
pins, and an exact retained lock. No shared Mere source or machine-local source
patch is used. Fresh branch-head resolution remains a separate supplier issue.

2026-10-23: the first clock candidate, seed `1792779701043394300`, passed the
ordinary ecology/placement preflight. Three ordinary ticks accepted five
food-directed steps. `inspect_foraging` reconstructs each exact pending native
request and verifies its living-sense ray; all five pass. Native replay reaches
`54795c56a1b4a43d`. All four retained older body/foot/whole-site histories still
replay to their recorded hashes.

The default camera followed critter 6, whose bounded routes were refused, and
its first recording has only one resting frame. That is retained as an
insufficient motion receipt. A second capture follows walker 19 in the same
seed. The complete native save is identical between the two captures: changing
the followed body changes the picture only. The followed receipt contains 24
actual GPU frames, 17 with supported swing, zero projection failures, zero
visible native box intersections and zero developer world intents. The WebP
preserves the recorded pixels and uses 200 ms per frame for inspection; raw
elapsed times and PNG hashes remain beside it.

The first run logs 13,487 native spans under the 16,384 cap. Its 5,445 eye/cache
reads total 310 ms (median 58 microseconds), and 7,131 food ray checks total
86 ms (median 12 microseconds). Cold Places remain dominant: 23 derivations
total 33.99 s, against 158 ms for 44 terrain-volume builds. These are CPU wall
spans across founding, forecasts and three ticks, not GPU timings or frame rate.

Final receiving checks on the recorded supplier cohort pass 129 cases: the 50
runtime/placement cases above, 27 isometer-space cases including raised outdoor
sight, and 52 host cases. Six existing profiling/headed cases are ignored. The
source, exact lock, optimized binary hash, failed attempts and compact evidence
remain in the named receipt folder under the 100 MB cap.

## Remaining limits

Terrain sight and native sensory availability now bound food eligibility.
Other bodies do not occlude these terrain rays; recognition, light, field of
view, smell and remembered food remain separate work. Range uses the existing
foraging search bound; sensory tissue area does not yet scale acuity.

The earlier sensing world recorded 61 refused routes out of 66 attempts. New
physical rules now require a complete native route before selecting food, and
the provider reuses that physical search. The 256-node bound remains; failure
does not establish global unreachability. Some statically supported bodies
still cannot swing a foot or reach a meal on their local terrain. The
[foot-placement follow-up](2026-10-22_foot_placement_plan.md#weight-shifting-follow-up-2026-10-23)
connects support-guided body transfers, nearby sole targets and shared gait
witnesses. Richer turning and connected founding placement remain separate
work; negative bounded searches do not establish global unreachability.
The capture qualifies three ticks, not long-term ecological balance or
force-bearing locomotion.
