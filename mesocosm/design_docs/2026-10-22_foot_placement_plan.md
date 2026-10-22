# Foot placement in ordinary play

Date: 2026-10-22. Status: implemented and captured in the optimized playable build.

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

Native turning uses a conservative swept envelope and can refuse on slopes;
the presentation snaps an admitted turn rather than sliding planted soles.
The chain convention remains the existing paid upper/lower/foot graph, not
arbitrary anatomical-joint solving. Only one foot swings at a time. Support
uses the declared document's volume centre as a geometric proxy, not posed
mass redistribution, force balance or dynamic weight transfer. Terrain
clearance is checked at the same 128 quantized phases that are rendered;
this does not qualify unsampled continuous trajectories or another engine's
physics. Alternative body comparison modes keep declared rest volumes.

The images still show long block-shaped bodies on thin links, and the terrain
pattern competes with their silhouettes. This change qualifies shared foot
geometry and a supported gait, not B2's varied inherited shapes, B4's local
blend radii or B5's muscle/load constants.

## Delivery locations

Work is on main in `Code/repos/isocosm`. The existing
`Code/worktrees/isocosm-playable` checkout and its `mesocosm/target` are reused
for Mesocosm's runnable Windows build. The existing named Cargo home
`C:/t/cargo-homes/isocosm-foraging` is reused for the same offline dependency
graph. These remain owned by that playable/rebuild workflow; this task creates
no isolated worktree, target or Cargo home. Eponym's three constructors are
updated for the shared optional feet field; headed qualification here covers
Mesocosm, not Eponym.
