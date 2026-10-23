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

Receipts: `Code/testing/mesocosm/senses-foraging-2026-10-23/`, total retained
cap 100 MB. Reuse the existing playable target and named foraging Cargo home.

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

## Progress

2026-10-23: traced the native choice, native part/system, geometric sight and
terrain cache paths; implementation started.

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

The first world also records 61 refused routes out of 66 attempts, all with
`no supported route to chosen food within the saved node budget`. The 256-node
route bound, terrain/support and occupancy admission remain authoritative;
this receipt does not establish whether each refusal is globally unreachable.
Route quality and choosing reachable visible meals are the next physical
foraging problem. The capture qualifies sensing and actual walking for three
ticks, not long-term ecological balance or force-bearing locomotion.
