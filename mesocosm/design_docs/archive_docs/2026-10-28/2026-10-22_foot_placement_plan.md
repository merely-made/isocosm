# Foot placement in ordinary play

Date: 2026-10-22. Status: original placement, bounded weight shifts and
intermediate stances qualified, 2026-10-23; remaining bounds are recorded below.

The previous [body appearance receipt](2026-10-22_body_appearance_plan.md)
showed moving roots but no ordinary foot swing. Native whole-body placement
could use a trunk corner as its sole support, leaving the real chains hanging.
Founders, arrivals and offspring searched the same small central lattice.

## Authority and scope

Mark requests actual foot placement and support on slopes, followed by a fresh
unchosen walking capture, and asks about overlap, facing and apparent scale.
He also explicitly permits revising earlier decisions as play teaches us more.
This implements geometric articulation through the existing native admission,
terrain, part graph and food-directed choice machinery. It does not introduce
an unmeasured muscle-force or load constant. The body-appearance choices stay
in their existing record; no case numbers are assigned here.

Isometer core owns shared integer chain geometry. Native spatial poses carry
admitted foot placements; native clearance and rendering consume those same
placements. Mesocosm owns placement/search policy and disposable interpolation.
Existing saved worlds retain their versioned rigid-placement behaviour.

## Done conditions

- Paid upper/lower/foot chains reach actual solid, dry terrain. Slope support
  comes from those feet, with unreachable or obstructed stances refused.
- Native occupied geometry and visible limb placement agree; visual movement
  cannot borrow clearance from the old rigid rest boxes.
- Walking remains driven by native needs and nearby eligible food. A fresh
  unchosen play run shows intermediate planted and swinging feet.
- Founding, arrival and offspring placement reuse the same support machinery.
  Measure spacing, orientation and body/window dimensions, then correct the
  central packing/facing faults without shrinking only the rendered bodies.
- Controls cover slope support, blocked/unreachable feet, severed chains,
  body collision and save/replay compatibility. Report limitations honestly.

## Receipts

Retain compact test/build/replay summaries, an unchosen native trace, actual GPU
viewport frames with contact counts, and a preview made from those frames under
`Code/testing/mesocosm/foot-placement-2026-10-22/`. Total cap: 100 MB. Preserve
the previous rich-bodies receipts unchanged. Reuse the existing playable Cargo
target and named Cargo home; no isolated worktree or build cache is needed.

## Implementation

Native `Pose` now carries optional body-local fixed-point sole centres keyed
by paid foot part IDs. New generated templates use version 4. Empty foot maps
retain the older serialized pose and rigid rules; replay does not redeclare a
world. Founding, arrivals and births carry the complete admitted pose.

Isometer core extracts the existing upper/lower/foot convention and solves
fixed-length chains with integer geometry. Native collision and the default
smooth renderer consume the same link capsules, pad and ankle connector.
Broad boxes still reject other bodies conservatively. Terrain checks use the
actual capsules, avoiding false shin collisions in empty box corners.

Grounding fits soles to dry terrain and searches a reachable root height.
The volume-weighted document centre must lie over the convex hull of actual
bearing terrain cells. A belly contact cannot substitute for paid feet.
Walking lifts one foot at a time while the others remain fixed in world
space, checking reach, terrain and support at 128 shared, quantized phases.
This is geometric balance, not muscle-load or momentum simulation.

Founders use distinct hashed anchors, varied quarter-turn facings and a
two-cell spacing policy stored in templates. Offspring and arrivals reuse
the same grounding search. Consumers use existing native food eligibility
to seek founding/arrival stands beside actual placed food; this does not
issue a feeding choice. Producers seat first regardless of entity ID order.
The food stand search shares one 625-candidate budget across its anchors.
The existing needs/food choice then uses a bounded priority route search.

The host retains explicit native admitted-step history, rechecks changed
terrain/body cuts and draws exactly the checked foot targets. Terrain
volumes and clearance results are reused between unchanged frames. Alternate
declared-volume comparison views retain their rest geometry.

## Qualification findings

The first unchosen run is deliberately retained as a failed receipt. It
showed six supported feet on a slope but zero walks: independently scattered
founders left food 98 cells away, beyond the native 32-cell search. That
finding led to the food-aware placement above. Its repeated `demo 1` actions
also issued ordinary player nudges; the next capture uses host-only pause
and single-step controls, with zero world-changing developer intents.

The second clock-sourced run accepted seed `15553628298003140480` at
candidate 9. It also had zero walks, then migrated immediately: its compatible
producer was in another site. The old structural startup check qualified a
diet somewhere in the world rather than an actual reachable placed meal.
Play startup now checks native food eligibility and occupied-box distance
against actual same-site admitted bodies before accepting its first eligible
candidate. This checks a starting condition, not a selected action or gait.
The failed trace and its six-foot slope picture remain in `guided.*` and
`motion-guided/`. The next run uses fresh output names.

The following first eligible world, seed `175745444879347014` at candidate
46 of an explicitly bounded 128-candidate search, chose eight hungry walks
and admitted none. Direct neighbouring-pose checks found the same first-sample
collision in every direction. When ankle and hip were level, the old thigh
pole became collinear with the target: a tiny foot lift flipped the knee
from above to below the terrain. The shared solver now gives its bend guide
a stable upward component. A fixed-point regression covers continuity through
that pose; no paid link length is stretched. The ordinary-walking test also
now counts accepted `Admission::Step` transitions, not requests that may have
been refused. `preflight.*` retains the failed observation. The configurable
default candidate cap is now 128, the existing maximum, because the new
physical-food starting condition can exceed the previous cap of 32.

That world's units are 1 mm per cell. Its walker is 40 mm long, its producer
74 mm, and the site is 2048 mm across. The admitted physical window is
196 mm wide and the camera shows a close-up. The apparent giant creatures
come from the narrow view, not a different render-only scale. Existing
`--slab` and camera options control framing.

The after-fix repeat of the originally clock-sourced seed is `knee.*`.
It takes two accepted native food-directed steps, with zero developer
intents, and admits one placed offspring by tick 8. The 24 actual GPU
viewport frames include 21 frames with simultaneous planted support and a
swinging foot, with zero projection failures and no intersections between
the visible bodies' native part boxes. The two bodies face Half and Quarter.
The walker's six feet occupy terrain heights 219, 220, 221, 223 and 228 mm.
`movement-knee.webp` packages unchanged frame pixels at 200 ms per frame;
measured times and source hashes remain in the frame records and summary.

That seed's walker is 40 mm long, producer 70 mm, site 592 mm across and
physical window 188 mm wide. Site size varies between seeds. At tick 8 the
walker is alive but travelling, so its old site's historical pose correctly
withdraws; this is not another failed foot placement. The native regression
checks that distinction. The preliminary capture uses the documented release
package opt-level-0 iteration overrides; optimized qualification follows below.

The consolidated check run passes 481 tests, with 24 existing ignores:
264 native library, 43 core geometry, 50 renderer, 51 host, 26 runtime and
47 selected native/runtime integrations. It includes the original slope,
riser, support, severance and blocker controls, the ordinary hungry-world
regression, the knee-continuity control, and corrected accepted-step counting.
Exact commands and build boundaries remain in the receipt logs.

The final normal release build passes the same two-step scenario with the
controlled body still physically placed. Both dimetric-south at `--slab 40`
and dimetric-north at `--slab 28` record 22 supported-swing frames out of 24,
zero projection failures, two distinct facings and zero visible native-box
intersections. The north view reveals the legs more clearly; its unchanged
frames are packaged in `movement-north.webp`. `overview.png` uses the same
tick-two save at `--slab 56`. All captures use host pacing only, with zero
developer intents. Measured frame times remain in the respective recorder
directories; the WebP previews use uniform 200 ms playback.

Native replay matches the final tick-two hash `0137807de2708f28` and the
eight-tick hash `6bcc647575ea1395`. Historical version-three controls retain
`e922048bc4f566c3` and `f5767ede7effce0f`. The final pictures and longer trace
therefore distinguish actual native departure from a placement failure.

## Remaining limits

The 2026-10-23 follow-up below addresses the straight root path, fixed foot
order and nominal-only foot columns. Its qualification is tracked separately
from the original receipt in this section.

Native turning uses a conservative swept envelope and can refuse on slopes;
the presentation snaps an admitted turn rather than sliding planted soles.
The chain convention remains the existing paid upper/lower/foot graph, not
arbitrary anatomical-joint solving. Only one foot swings at a time. Support
uses the declared document's rest-volume centre as a geometric proxy. Root
transfers are kinematic; posed mass, actuator capacity, force balance, friction
and momentum remain separate work. Terrain
clearance is checked at the same quantized phases that are rendered: 128 for
the original gait, and at least 128 or 16 per moving foot for explicit strides;
this does not qualify unsampled continuous trajectories or another engine's
physics. Alternative body comparison modes keep declared rest volumes.

## Weight shifting follow-up, 2026-10-23

Mark requests weight shifting and better foot placement through the existing
limb and support machinery, followed by an unchosen moving capture.

### Phases and done-conditions

1. Reuse the actual dry bearing-cell polygon and paid limb solver to plan a
   body shift before lifting a foot. Share deterministic integer trajectory
   data and sampling; native validation and the renderer must consume that
   same recorded plan. Retain old saved worlds' omitted-field semantics.
   Controls must show a previously unbalanced swing becoming supported, keep
   planted feet fixed, and reject forged, obstructed or unreachable motions.
2. Let grounding search nearby dry foot columns within explicit saved bounds,
   reusing the existing height cache, capsule clearance and support checks.
   Founding, walking, births and arrivals use the same placement machinery.
   Controls cover a nominal foot column that fails beside a valid foothold,
   true gaps and water, and body obstruction. Route caching includes the new
   physical policy and records complete gait witnesses before choosing food.
3. Build with the qualified dependency lock and normal release optimization.
   Run the first eligible unchosen world, capture real intermediate frames,
   verify weight shifts and planted contacts, and replay retained old saves.
   Record remaining refusals and bounded costs without relaxing clearance,
   support or route budgets to obtain a capture.

Receipts: `Code/testing/mesocosm/weight-shifting-2026-10-23/`, total retained
cap 100 MB. Reuse Mesocosm's existing playable checkout, target and named Cargo
home. Preserve earlier receipts. No new worktree or build cache is needed.

### Findings

2026-10-23: `embodiment/swing.rs` moves the root linearly throughout a fixed
part-ID foot order. `articulation::support_at` already builds the convex hull
of actual dry pad contacts, but only tests the body volume centre against it.
`articulation::ground` tries the nominal sole columns and root heights; it does
not search nearby sole targets. `section/smooth/motion.rs` samples native feet
but separately interpolates the root. These must be connected by an explicit
native gait witness, rather than independently planning a visible trajectory.

Ownership: isometer-core carries product-neutral fixed-point gait data and
sampling. Native Isocosm owns terrain/body queries, support-guided planning and
admission. Mesocosm carries accepted witnesses to its disposable animation.
The existing volume-centre balance proxy remains the support law; this work
does not assign muscle forces, friction coefficients or momentum constants.

### Progress

2026-10-23: traced the four retained local failures, existing support polygon,
paid-chain geometry, route search, native publication and renderer sampling.
The new optional saved `Templates.walking` policy declares a four-cell body
shift bound, two-cell foothold search radius, three-cell maximum lift and 256
candidate-posture limit. New physical founding opts in. Old saved rules omit
the policy and retain the original grounding and swing semantics. Its saved
`clearance` flag selects a vertical rise before horizontal transfer, followed
by lowering after crossing. Omission retains the earlier straight transfer and
its one-cell trial-rise limit. Strides record the same flag and admission
requires it to match the policy.

`isometer-core::legs::Stride` records foot order, fixed-point body-transfer
knots and clearance. Native support-guided planning tries the actual remaining
bearing polygon before each takeoff, then validates the complete plan through
the existing limb capsules, terrain and current-body checks. The geometry
witness travels through the accepted native step and Mesocosm admission record
to `Motion`; the renderer samples that witness for both root and feet.

Nearby sole targets reuse `Grounding`'s height cache. Per-limb reach and capsule
clearance filter the candidates, then the existing whole-body support check
admits a bounded posture. Founding, routes, births and arrivals use this path.
The cache key includes the walking policy, and native publication rechecks the
recorded path and stride against current occupancy.

Six core controls and thirteen native embodiment controls pass, including a
loaded corner that cannot lift in the old fixed order, a missing nominal
foothold beside dry support, world-fixed bearing feet and rejected forged or
blocked trajectories and the order of rise, crossing and lowering. Optimized
host and runtime qualification is in progress.

The first optimized pass accepts eleven food steps across five critters in
unchosen seed `72173207038511016`, the thirtieth startup candidate from
clock draw `1792799530492473600`. North and south captures have identical
complete native saves. The first camera hides feet behind a slope; south
reveals them. The retained steep sensing-world case still fails: two-cell
transfers cannot find a bearing polygon, and three/four-cell straight transfers
hit part 6 against an uphill voxel at their first sample. That evidence calls
for raising the root before crossing, with every planted sole kept fixed and
paid reach rechecked. Final qualification below uses that ordered transfer.

### Final qualification, 2026-10-23

The normal release build repeats the actual accepted seed from the original
unchosen run. Complete founding data, terrain, food, anatomy and initial
placements match after removing only `Templates.walking`. Three ordinary
ticks now accept seventeen sighted food steps across seven critters, against
eleven across five with the first straight-transfer policy. All seventeen
complete route witnesses pass native verification, ten contain lateral root
transfers, and all seventeen begin with feet at different terrain heights.
The final native hash is `b629dab76e9ad27d`.

`clearance-moving.webp` packages twenty-four actual GPU frames from the south
camera, including seventeen supported-swing frames and twenty body-transfer
frames. There are zero projection failures, endpoint native-box intersections
or developer world intents. Pixel changes occur within unchanged native cuts.
The renderer reads the accepted native trajectory for root and feet.

129 receiving controls pass: six core, thirteen native embodiment, fifty-four
host and fifty-six runtime/placement/senses/route controls. Six existing
profiling/headed cases are ignored and one multi-seed search is filtered.
Seven retained native histories replay exactly, including the first new-policy
capture whose omitted clearance flag retains its earlier straight transfers.
Eponym's mechanical constructor update is included; its independent check stops
before source compilation at its existing stale lockfile.

At the same first pending cut, the census finds seven complete routes among
twenty-three nonempty searches, against five in the first pass. Sixteen searches
still have no route, eight exhaust 256 nodes. The retained steep sensing-world
pose still has no admitted adjacent transfer: trials at shifts one through four
fail paid reach. These are bounded planner refusals, not proofs of global
unreachability. Multi-stage footholds, richer body articulation and turns remain
possible follow-ups; this work does not stretch limbs or relax support.

The final capture logs 3,463 native spans below the 131,072 cap. Sixteen cold
Places builds total 48.04 s; forty-seven route searches total 23.24 s, median
143 ms and maximum 4.03 s. Walking planning totals 11.91 s over 757 calls;
checking totals 6.97 s over 533 calls. Nested spans overlap. The capture overlaps
a retained-case diagnostic and other workspace builds, so these CPU wall spans
are a cost census rather than a comparative frame-rate benchmark. Exact source,
locks, binary hashes, the rejected straight trials and raw frames remain in the
named receipt directory under 100 MB.

The original 2026-10-22 pictures qualify shared foot geometry and supported
gait. Body appearance and actuator/load rules are tracked separately in the
[body appearance plan](2026-10-22_body_appearance_plan.md); this follow-up
qualifies placement and motion.

## Intermediate stance follow-up, 2026-10-23

Mark requests intermediate stances within paid reach for the retained steep
case, followed by an unchosen moving capture. This is implementation within
the existing limb/support model. Responsa's path lookup finds no governing
decision on the walking module; the case record and generated manifests are
unchanged.

Done-conditions:

1. Reuse the existing stride representation and body/sole sampling. Retain
   useful world-space contacts in supported arrival stances and bound their
   number with an optional saved policy. Omission preserves canonical arrivals.
2. Search connected stances through the existing nearby dry-target cache,
   support polygon, paid limb solver and narrow clearance checks. Validate
   the complete proposed motion at every rendered phase. The retained pose
   and terrain get a regression; unsupported, stretched, blocked and malformed
   trajectories remain rejection controls.
3. Qualify the optimized build, replay retained worlds, run the first eligible
   unchosen seed and capture actual intermediate movement. Record any remaining
   refusals and bounded costs. Keep native admission authoritative and present
   its exact accepted trajectory.

Receipts: `Code/testing/mesocosm/intermediate-stances-2026-10-23/`, total cap
100 MB. Reuse the existing playable checkout, target and named Cargo home.
The checked-in steep fixture is extracted from the retained native genesis
and exact pending-cut pose; live occupancy is checked again by the diagnostic.

The retained terrain/limb regression admits all four neighbouring
root moves while keeping the soles fixed. The old canonical destinations
still reproduce the paid-reach refusal. This is an endpoint-choice problem:
the terrain-only neighbour query discarded the current contact configuration.

New `Walking.stances` defaults to four supported arrivals per neighbour,
including the canonical stand, with the existing 256-posture limit bounding
trials. Routes distinguish foot layouts at the same root/yaw. The body-transfer
planner can retain its current supported root before a foot moves. Older rules
omit `stances`, keep canonical neighbours and retain their previous route keys.

An exploratory temporary-landing protocol and separate stance solver did not
resolve the forced canonical destinations; their sources and failed probes
are retained in the receipt. They are removed from the implementation. The
existing shared stride already represents the accepted planted-body transfers.
The occupied-scene diagnostic admits seven variants across those same four
neighbouring roots, including the uphill root. Four retain all world-space
soles; three use one foot stage. The old saved world's rules and history remain
unchanged: this diagnostic evaluates the proposed policy at its exact retained
choice point.

Qualification passes 126 selected controls: fifteen native spatial controls,
one route-state identity control, fifty-four host controls, twenty-six runtime
controls and thirty integration controls. Six existing headed/profiling cases
are ignored; one older multi-seed integration case is filtered. Receiving
release type checks cover all host/runtime targets. Eight older native traces
replay to their recorded hashes.

The normal optimized build runs `--play --unchosen-seed` with clock origin
`1792810898425799800`. Its existing structural/placement preflight admits
candidate 71/128, seed `3577203618348961378`; no motion criterion selects the
seed. Three ordinary ticks yield fifteen food-directed steps across nine
creatures and native hash `2ed329360b247f4f`. Exact native choice-point checks
verify every accepted step's sight witness, complete route, paid limb motion
and current occupancy; no recorded route is refused.

Twenty-four actual frames include thirteen supported-swing frames and twenty
planted-body-transfer frames. Projection failures, endpoint native box overlaps
and developer world intents are zero. An isometric-north recapture of the same
seed makes the soles clearer than the hillside-backed dimetric-south view.
Its native save is identical; its twenty-four frames include fifteen supported
swing frames and twenty-three planted-body-transfer frames. The preview uses
200 ms per captured frame; raw frame times are retained.

At the first choice point, nine of thirteen reports contain routes, three
return before physical search and one exhausts the unchanged 256-node bound.
The latter remains unresolved, not proof of global unreachability. Body centre
is still the existing rest-volume geometric proxy. Posed mass, force, friction,
actuator capacity, momentum and richer turning remain separate work.

The original capture records 3,639 spans below its 131,072 cap. Startup founder
placement totals 50.53 s across 1,140 calls, including rejected candidates.
Seven cold Places builds total 46.92 s. Twenty-nine route searches total 2.78 s,
median 29 ms, maximum 1.56 s. Walking planning totals 0.58 s over thirty-nine
calls; checking totals 1.38 s over 123 calls. Nested spans overlap. The capture
overlaps native diagnostic/replay work and other workspace builds, so this is
a cost census rather than a frame-rate or cross-seed speed comparison. The
initial process peaks at 2.72 GB working set. Source, locks, binary hashes,
failed exploratory probes, both captures and exact witnesses are retained
under the 100 MB receipt cap.

## Supported turning follow-up, 2026-10-24

Mark agrees to connect supported quarter-turns and steering next. This is
implementation of the existing limb/support model. Responsa path lookups find
no governing decision on the affected walking, stride or food-route modules.
The case record and generated decision files are unchanged.

Shared Isometer core owns the deterministic turn sampling alongside its
existing stride. Native Isocosm owns candidate placement, paid reach, terrain,
support, live occupancy and admission. Mesocosm presents the accepted sampled
orientation and soles. An optional saved walking flag enables the new path;
omission preserves earlier saved policies.

Done-conditions:

1. Construct supported quarter-turns by moving feet to a shared stance,
   pivoting over its planted soles, then using the existing stride planner to
   complete the step. Validate every displayed phase against paid chains,
   actual dry support, narrow terrain and current occupied geometry.
2. Connect turns to the existing food-route search, with facing as a preference
   between equally short routes. Retain legitimate sideways movement and all
   existing route/work bounds. Test supported pivots, blocked turns and an
   obstruction that requires a change of facing.
3. Present the same admitted turn and collect actual pivot/sole evidence.
   Run the first eligible unchosen seed, verify its exact native food routes,
   capture movement, and replay the retained older native histories.

Receipts: `Code/testing/mesocosm/turning-2026-10-24/`, total retained cap 100 MB.
Reuse the existing playable checkout, target and named Cargo home. Record
remaining refusals and measured costs. Startup optimization remains a later
slice.

The implementation extends the existing stride stages with optional explicit
world-space sole targets and relative body orientations. It reuses ordinary
foot-step planning between small supported pivots, trying eight or four
pieces for a quarter-turn. Body-only stages retain every sole. The same
fixed-point rotation and sampled trajectory drive native paid-chain checks
and display. The sampler retains ordinary stage resolution. Candidate stances
and foot-step planning share the declared posture budget; complete motion
retains the existing clearance-work and sixty-four-stage bounds.

A first, separate whole-quarter pivot format passed the toy quadruped and
slope checks but failed the generated-body render fixture: the longer bodies
could not keep one stance through ninety degrees. Its source and failed
receipt are retained. The delivered approach instead merges turning and
repeated foot placement into the existing stage sequence; old fields remain
omitted in ordinary saved strides.

Foot support uses the actual rotated pad footprint against dry terrain
columns, with the existing bearing polygon and volume-centre law. Rigid-part
clearance remains conservative rotated bounds. Contacts retain the existing
sole-centre representation; independent ankle yaw and friction are unmodelled.
The optional saved `Walking.turns` flag defaults on for new worlds. Omission
preserves older policy, route ordering and stride serialization.

Routes can include supported adjacent quarter-turns and turns at one root.
Facing breaks ties between equally short routes. A longer route may still
lose to a short sideways step; the route-node cap remains 256. Foot layouts
remain distinct route states when supported turns are enabled.

The first unchosen draw admitted candidate 10/128, seed
`14417468444982103644`. Its first three ticks produced forty-one verified food
steps across twenty-three walkers, hash `1a60465b3560d3ef`, but no turn. The
twenty-three actual frames show supported walking and zero projection failures.
This remains the initial failed turning demonstration, not final acceptance.

The live refusal probe found actor 41's fixed pivot root outside the bearing
polygon. The follow-up reuses polygon balance during rotation, preserves
usable current contacts and permits a lower torso stance within `shift`.
Turning alone may lower the root by at most that declared deviation; ordinary
saved stride restrictions are unchanged. Narrow terrain, current bodies, paid
reach and support still validate the complete trajectory. The actor's exact
terrain, anatomy and two admitted static turn goals become a regression;
the recovered trajectory includes a torso shift during the pivot. The final
capture retests this same unchosen genesis with presentation following actor
41. The initial capture, exact sources, binary hashes and failures remain in
the receipt directory.

Status: 138 selected controls pass: eight shared limb/sampler, nineteen native
spatial, fifty-five host, twenty-six runtime and thirty integration controls.
These include two ground heights, a blocked turn, an obstacle requiring
turning in a food route, generated anatomy in the renderer, the retained steep
stance and the first unchosen support refusal. Six existing ignored tests
remain ignored; one older multi-seed integration is filtered.

The repaired three-tick run of the exact same genesis has hash
`ace920c6cd43b872`, forty verified sight-directed food steps across twenty-three
walkers and two admitted turns, by actors 65 and 60. Every accepted food step
was rechecked at its exact native choice cut, including its complete route
and paid stride. No recorded food route was refused. The first-cut census
still has one actor exhausting the 256-node limit and four returning before
search, among twenty-eight reports.

Following actor 41 did not show the two remote turns. A presentation-only
recapture follows actor 65 and reproduces the identical complete native
history. `turn-view-moving.webp` contains twenty-four actual frames, including
six with supported swings, twelve with torso shifts and one with a planted
pivot. Projection failures and reported endpoint-box intersections are zero.
This records the native sole-centre and paid-chain model; it does not qualify
independent ankle articulation or continuous moving-body contact dynamics.

`balanced-profile.json` records 9,248 spans below the 131,072-span cap. The
repaired process took 67.52 seconds with a 624.3 MB peak working
set. Thirty-one cold terrain/place builds cost 22.76 seconds; eighty-two route
searches cost 18.78 seconds; 980 stride plans cost 16.08 seconds. Nested spans
overlap. The initial failed run had other replay work running, so these are
cost censuses, not a controlled before/after speed comparison. Startup and
repeated-work optimization remain a later slice. All nine retained older
native histories replay to their exact recorded hashes with the final binary.
`qualification.json`, the source archive and executable hashes identify this
receipt; the headed claim is limited to the captured Mesocosm run.

## Delivery locations

Work is on main in `Code/repos/isocosm`. The existing
`Code/worktrees/isocosm-playable` checkout and its `mesocosm/target` are reused
for Mesocosm's runnable Windows build. The existing named Cargo home
`C:/t/cargo-homes/isocosm-foraging` is reused for the same offline dependency
graph. These remain owned by that playable/rebuild workflow; this task creates
no isolated worktree, target or Cargo home. Eponym's geometry constructor is
updated for the shared optional stride field; headed qualification here covers
Mesocosm, not Eponym.


---

*Archived 2026-10-28.* Cut, with the other active plans, into a new set: what was
still open, deferred or a follow-on here now lives in `2026-10-28_bodies_plan.md` (in the product's
`design_docs/`), which cites this file by line. This copy is history: its
status lines, receipts and claims are as they stood, not current.
