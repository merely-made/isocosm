# Bodies

**Status:** 2026-10-28: cut from `2026-10-22_body_appearance_plan.md`,
`2026-10-22_foot_placement_plan.md`, `2026-10-23_senses_foraging_plan.md` and
`2026-09-26_body_binding_plan.md`, which are archived in
`archive_docs/2026-10-28/`, plus the body-related remainders of the
presentation, place graph, sim and execution plans. Built: tissue colour through
the smooth path, native recipe proportions with role-based selection, native
chains, paid upper/lower/foot chains with weight shifting, intermediate stances,
supported turns, posed-mass balance, terrain sight rays and a complete-route
requirement for food. Open: per-connection blending, the strength phase, the
full-density calibration, richer senses, and the whole body-binding module, which
is documented but not built.

**For:** bodies in the world: how they look (appearance), how they stand and
step (footing), what they sense (senses) and how a sim entity's body is bound to
a physics body (binding). Design core: isometer owns the integer chain geometry
(the paid upper, lower and foot chains and their sole poses), and native
admission owns the support check that stand, route, birth and occupancy
admission must pass (doctrine Bodies 2: geometry is isometer's, physiology the
sim's). Not here: tick latency and cold route, startup and Places search cost
(`sim-execution`, its Speed bars); the detail ladder, far-rung marks and the
tenant path (`orthographic-voxel-presentation`, including the far-rung function
tint question); effects and magic (`functional-generation`); the world store's
lifts (`place-graph-engine`).

Receipts from this plan's runs go to one file,
`archive_docs/receipts/2026-10-28_bodies_plan_receipts.md`, never into this plan (DOC_POLICY
§8). Evidence that matters is also a test in the repository or a line in
Findings.

## Decisions (Mark, 2026-10-22, unnumbered)

Each was put as a card with its evidence; the option Mark chose is quoted.

- **Chains.** "Accept chains in native recipes." The jointed sheet's paid
  chains (legs with feet, stalked leaves, note dated 2026-09-04) reach play;
  admission still prices each step. Reverses no live case: the refusal was a
  phase boundary of the archived VB2 phenotype work, not a ruling.
- **Blending.** "Soft at joints, crisp at plates, shells and mouths." One
  blend radius per connection, carried by the part graph.
- **Scaling.** "Adopt both: actuation by area, mass by volume." Feet support
  what their cross-section can bear; long slender links reach but carry less.
  The constants are Mark's to number.
- **Order.** "Materials through the smooth path first." Then the generator
  uses the sheet's shapes, then chains, then proportions.

## Constraints
- Any fork with more than one defensible answer returns to Mark; a phase that
  shows something is verified with a picture where visible; inheritance variance
  over shape choice is kept. Capacity-scaled limb templates are only 1x and 2x
  half-extents; multi-function authored templates retain declared allocation;
  omitted bounds and fixed presets retain historical dimensions. Source:
  `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:79-83, 147-155`.
- The strength constants are Mark's to number; nothing is built from them until
  they come back numbered (Decisions above, Scaling).
- No new shape name is coined (doctrine, Naming 3).
- A change to bounded body search keeps every route, posture, work and paid-stride
  budget; stance states are not pruned. Source:
  `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:81-86`.
- A refused chain step keeps the biology and withdraws the picture, as a refused
  placement does today.
- Binding (ruling 346 to 352): bodies only, on a borrowed shared `BodyWorld`;
  the module names no product type and adds no dependency to conatus; a moved
  body keeps its id and a shape revision change respawns; no `BodyId` is
  serialized. `TactileWorld` stays advisory; native integer placement admits
  saved poses.

## Phases

### Appearance

#### BO1. Blending per connection (was B4)
Each connection in the part graph carries a blend radius, soft at joints and
crisp at plates, shells and mouth openings. The renderer reads it; no part
changes its envelope. Not built. Gradient-based blending (Gourmel 2013) stays an
option "if bulges remain, as its own later phase".
- Done when: "a frame shows a mouth opening and a plate edge as crisp against a soft joint on the same body; the implicit renderer's tests cover both settings; cost measured against the current single blend on the same frame and reported, not judged."
- Needs: part-graph connection data; the implicit renderer (`isometer-render`)
- Rulings: Mark, 2026-10-22: "Soft at joints, crisp at plates, shells and mouths."
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:43-44, 126-136`

#### BO2. Strength by area, mass by volume (was B5)
A limb's actuation scales with its cross-section and a body's mass with its
volume, so a long slender link reaches but carries less and a stout one bears
more. Built without constants: posed-mass balance (`Walking.balance`) and adult
mass by voxel volume. Unbuilt: area-scaled actuation and weight-bearing stance
admission. This phase proposes the constants with their measured range and builds
nothing until Mark numbers them. Force and load dynamics stay out: muscle force,
actuator capacity, force balance, friction, momentum, per-foot reaction forces.
Root transfers are kinematic.
- Done when: "the gait refuses a stance whose feet cannot bear the body and chooses another; the same seeds still reproduce; a proposal of the constants with evidence has gone to Mark and come back numbered."
- Needs: BO3 (calibration evidence); Mark numbering the constants
- Rulings: Mark, 2026-10-22: "Adopt both: actuation by area, mass by volume." "The constants are Mark's to number."
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:46-48, 138-140, 288-296`; `archive_docs/2026-10-28/2026-10-22_foot_placement_plan.md:166-170, 384-386`

#### BO3. Full-density calibration gate
Walking tissue fills only 0.055-0.350% of current-part capacity, unlocalized
entity inventory is not in part loads, and pad contacts are summed per foot as
quasi-static readings, not per-foot reaction forces or a strength law. Growth
beyond the founding mass in a walking pose is not established. The posed-balance
draw is short and "cannot calibrate strength constants". A per-foot force split is
not invented here.
- Done when: not stated ("the fuller-body calibration gate remains open"); owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:196-209, 280-286`

#### BO4. Curved leaf and shell contents, and anatomy vocabulary
The captured plates still read as panels even with better proportions: flat
shell plates read as side panels and upright fronds as signs. Wanted: curved leaf
and shell contents inside the declared volumes; species likeness and surface
markings; shells, tapering, rounded forms and coherent surface patterns; a larger
finite voxel shape bank; a broader controls inventory, arbitrary topology edits,
chain variation, per-part generation traces and surface-pattern variation. Core
does not enforce whole-body self-intersection freedom. Population rendering cost
for generated anatomy is unmeasured. Flight, swimming, climbing and root
physiology stay application capabilities.
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:298-301, 342-346`; `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1708-1713, 1767-1773, 1838-1842`

#### BO5. Cross-site camera follow
Dev follow moves the camera while the scene still selects the played critter's
native site; one draw's final still is blank after the played critter changes
sites; another's first follow entered a journey. Cross-site following is not
qualified.
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:219-223`

#### BO6. Grown-body sampling cost
Long runs of grown-body sampling were stopped after about 24 minutes at high
working sets, so shorter cuts of the same worlds were used; shared-machine runs
do not qualify round latency or attribute the delay to one physical stage.
Startup still rejects many candidates for physical stands or nearby food
(eligibility unchanged). The latency bars themselves belong to `sim-execution`.
- Done when: not stated; owed
- Needs: `sim-execution` (Speed bars; its cold route and startup search items)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:172-176, 345-346`

### Footing

#### BO7. Articulation in native admission
Native joint limits and articulated occupied geometry must participate in stand,
route, birth and occupancy admission before a pose may lower a rigid trunk or
extend a limb outside its declared or cleared envelope. The paid
upper/lower/foot chain capsules now used in native admission meet part of this;
arbitrary joint graphs and declared joint limits are not met. The chain
convention is that paid graph, "not arbitrary anatomical-joint solving". Also open:
declared joints and limits, richer generated body plans, and ankle yaw (and
friction), which are unmodelled.
- Done when: not stated; the gap is stated as "must participate in stand, route, birth and occupancy admission before..."
- Needs: `BO13`'s boundary (native versus Conatus); `BO17`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:407-411, 569-573, 657-660`; `archive_docs/2026-10-28/2026-10-22_foot_placement_plan.md:164-167`

#### BO8. Gait: turns, stages and sampling
Mark, 2026-10-28, on quantized facing: "Except for things like maybe feet, which are automatically placed anyway, so might as well do quantized facing and then use that for the leaning and the, you know, the body weight and which weight is on the foot and, you know, to determine like if somebody has balance and whether or not they're gonna trip and all that stuff." A direction for this phase, not a ruling on its form: feet take a quantized facing, and lean, weight per foot, balance and tripping read from it.
Turning is conservative: a longer route may still lose to a short sideways step,
and slope refusals were not re-stated after supported turns landed. Only one foot
swings at a time; multi-stage footholds and richer body articulation remain
possible follow-ups, and this work does not stretch limbs or relax support.
Terrain clearance is checked only at the quantized rendered phases (at least 128
for the original gait; at least 128 or 16 per moving foot for explicit strides),
so unsampled continuous trajectories and other engines' physics are not qualified.
Continuous moving-body contact dynamics are not qualified; the sample qualifies
three ticks, not long-term ecology or force-bearing locomotion.
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-22_foot_placement_plan.md:164-174, 292-294, 454-498`

#### BO9. Controlled-intent and bench remainders
Broader controlled intents (only Idle intents and one-tick carves exist), contact
localization (per-foot contacts, impact forces, water interaction need evidenced
inputs), material reactions, transparency, sound and integrated cost. Bench D:
real generated anatomy through ecological projection with terrain and causal
effects together, transparent compositing, full memory accounting, scaled
ecological scenes, GPU timing (timestamp queries disabled), application working
limits. A three-way generated depth oracle remains open. Uptake is
organism-level, not contact trace. Natural-checkpoint and donor-removal have no
native acceptance.
- Done when: Bench D: "equivalent camera, poses and pixel scale accompany correctness, first-frame cost, median/p95/max work, mesh/instance uploads, texture/staging cost and cache activity. Carry the equivalent-pose yaw baseline and document the renderer's policy for coincident surfaces; count foreground coverage separately from background. Record the source revisions and benchmark population, including unique geometry."
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1922-2226`

### Senses

#### BO10. Sensing beyond terrain rays
Other bodies do not occlude terrain sight rays. Recognition, light, field of
view, smell, remembered food and learned recognition are separate work. Range
uses the existing foraging search bound and sensory tissue area does not scale
acuity; detection is any occupied part of food, with no light sensitivity or
field of view. Ordinary generated ecology leaves world and carried organ-system
maps empty even when a recipe has sensory cells; the default nervous network is
declared only for new physical founding and realized sensory networks only on
founders and developments, so older saved worlds are not regenerated. The
top-centre sight fallback should become sensor-part positions (`Process::Sense`
exists; Ground has no derived location for a living sensor part), and oriented
collision, facing, gait and deformability should refine the turning
cross-section. Stop rule: "prefer another causal join over declaring the current
encounter finished".
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-23_senses_foraging_plan.md:7-8, 49-55, 137-147, 195-198`; `mesocosm/design_docs/archive_docs/2026-10-28/2026-08-05_place_graph_engine_plan.md:1084-1109, 1136`

#### BO11. Complete routes and bounded search
A complete native route is required before food is selected. The 256-node bound
remains, and a failed search does not establish global unreachability; some
statically supported bodies still cannot swing a foot or reach a meal on their
local terrain. Bounded search refusals are not proofs: intermediate stances
recover the retained steep sensing world, but one first-cut search still exhausts
the bound. The sample qualifies three ticks, not long-term ecology.
- Done when: not stated; owed
- Needs: BO8; `sim-execution` (cold route search cost)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-23_senses_foraging_plan.md:200-214`; `archive_docs/2026-10-28/2026-10-22_foot_placement_plan.md:288-294, 380-386, 489-490`

#### BO12. Forager target deliberation
On the `playable-loop` branch (unpushed at 2026-10-23) the forager target comes
from a route search ahead of the prey scan; its cost is deliberation the A1
battery does not exercise, and no battery measures it. The bar, if any, is
`sim-execution`'s.
- Done when: not stated; owed
- Needs: the `playable-loop` branch landing; `sim-execution` (Speed bars)
- Rulings: -
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-10-19_sim_execution_plan.md:461-472`

### Binding

#### BO13. The body-binding module in conatus
Build the module in mere's `conatus`, generic over the consumer's key
(`BodyBindings<K>` with `reconcile`, `set`, `remove`, `body`, `key`). Documented,
not built; the VTT's earlier `isometry-runtime` binding table and event mirror
never reached the stack, and ruling 346 documents one binding shape, built when
Mark says.
- Done when: "the module exists in `conatus`, names no product type, adds no dependency to conatus, and stays under the 600-line ceiling; over a seeded draw of accepted sets and edits, a cold `reconcile` and the same history applied through `set` and `remove` leave the same keys, transforms and shapes; a key removed and added back never resolves to its old body, and an old `BodyId` resolves to no key; a duplicate key, and a body despawned behind the module, are refused before any world call; after a conatus refusal partway through an apply, the table names exactly the bodies the world holds; `key()` of an unbound body is `None`; no `BodyId` reaches anything serialized; and conatus's docs drop their interim wording in the same change." And, for any binding build: "`cargo check --workspace --all-features --all-targets` is green"
- Needs: Mark's build word; a mere lane; mere's T2 terrain collider (rulings 330 to 335, 349, 363), which is separate and replaces `TactileWorld`'s terrain half on its own schedule
- Rulings: 346, 347, 348, 349, 350, 351, 352; 299; 330 to 335, 363
- From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:222-284, 343-353, 363-364, 48-56`; `design_docs/archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:229-232`

#### BO14. Mesocosm adopts the module
`TactileWorld`'s critter half (map, `set_critter`, `clear_critter`, the critter
branch of `pick`) becomes the module; capsule lowering stays Mesocosm's; the
picking judgment ports. The old plan's key `OrganismId` is stale (the core was
deleted): key by the sim entity handle. The picking example named in the old
plan no longer exists, so its receipt must be re-established by a new test.
- Done when: "`TactileWorld`'s critter tests pass through the module and `t1_picking` reproduces its receipt: 22 judged stops, 16 ground, 4 critter, 2 nothing, the judgment agreeing bit for bit across two fresh runs."
- Needs: BO13
- Rulings: 346, 347, 348
- From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:297-305, 354-357`

#### BO15. The VTT adopts the module
When its first conatus consumer opens: tokens as conatus bodies, tiles lowered
through `BoardWorld::stand`.
- Done when: "the retired profile's four tests (`lib.rs:366-469`), ported to the board's lowering, pass against the module, and a token's conatus body and its drawn body stand at the same world point for every tile and elevation of a seeded draw of maps."
- Needs: BO13; a VTT conatus consumer; the key question below
- Rulings: 243, 299, 346
- From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:307-317, 358-362`

#### BO16. Finish documenting in mere (Appendix A)
The `engine.rs` doc comment, the composition plan's tactile row and C4 text, and
mere's index rows, and name the mere commit in this plan's status.
- Done when: "a mere lane has applied Appendix A at mere's main: the 'Body bindings' text of A.1 stands in the conatus engine plan's §1; the three passages read as A.2 gives them; mere's two index rows carry A.3's sentences, as mere's DOC_POLICY §6 asks; no binding code is written, the `Engine` doc comment being the only line under `crates/` the change touches; no live mere passage still puts the key-to-body table outside conatus, those A.4 lists saying only that a source's meaning is the profile's; and this plan's status and progress name the mere commit."
- Needs: a mere lane
- Rulings: 346, 352
- From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:329-339, 511-722`

#### BO17. Articulated and multi-part binding
A generic Conatus binding adapter, native joint state, articulated collision
admission, force-based actuation, weight balance and jointed native collision
admission remain unbuilt; the shared pose controller consumes native support
witnesses over admitted root poses. Also open: `Entity.skills` is keyed numerics
and `rules/body.rs` binds a required function to one live part, so storage and
multi-part binding (skills and abilities supported by several parts and
equipment) are not designed.
- Done when: not stated; owed
- Needs: BO13; BO7; BO2
- Rulings: 346 to 352, 804; 405
- From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:3-24`; `mesocosm/design_docs/archive_docs/2026-10-28/2026-09-22_sim_plan.md:186-190`

## Remaining limits
One merged list; each names the phase that owns it.
1. Force and load dynamics: muscle force, actuator capacity, friction, momentum, any strength law (BO2).
2. Posed mass at realistic density; growth beyond founding mass in a walking pose (BO3).
3. Arbitrary anatomical joints, declared joint limits, ankle yaw (BO7).
4. One swinging foot, conservative turns, unsampled continuous trajectories (BO8).
5. Bounded search: refusals are not proofs of unreachability (BO11).
6. Sensing: occlusion by bodies, recognition, light, field of view, smell, memory, acuity by tissue (BO10).
7. Readability: plates as panels, fronds as signs, crowded silhouettes, unbuilt blending (BO1, BO4, `orthographic-voxel-presentation`).
8. Camera: cross-site follow unqualified (BO5); alternative comparison body modes (voxels etc.) keep declared rest volumes.
9. Scope of evidence: receipts qualify two or three ticks on short unchosen draws, not generations, epochs, ecological balance, dense populations, arbitrary site sizes or an interactive frame budget; Eponym headed qualification is absent (Eponym is only mechanically updated for the optional stride field, and its independent check stops before source compilation at its stale lockfile).
10. Cost: cold-start and startup viability search (`sim-execution`, BO6).
11. Physical continuation: automatic resettling, gravity, aquatic and flight support, all-individual placement, and persistent waking that accounts for pose and food-target reads as well as bodies (`sim-execution E1`).

## Deferred and parked
- **Eponym motion tests**: eighteen of thirty-one fail at baseline (no standable fixture position, missing ledges, `InvalidStart`, archive decoding), before the binding query setup is reached, so query refresh does not certify those motion and save paths. Belongs to `eponym-overlay`. Ruling 804. From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:416-420, 481-487`.

## Questions for Mark
- **Strength constants**: the constants for BO2 are Mark's to number once the proposal with evidence reaches him. From: `archive_docs/2026-10-28/2026-10-22_body_appearance_plan.md:138-140`.
- **Build word for the binding module**: ruling 346 builds it "when Mark says"; BO13 to BO15 wait on it. From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:343-353`.
- **Binding key for the VTT**: `(map, TokenId)`, or the site `PlaceHandle` under ruling 243. From: `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md:307-317`.
- **Two bodies overlapping**: nothing in movement keeps two bodies from overlapping (a reading, not ruled); shared fixtures space bodies at native bounds instead (case 941). Whether movement should prevent it is open. From: `eponym/design_docs/archive_docs/2026-10-28/2026-09-25_eponym_overlay_plan.md:491`.

## Findings
- 2026-10-28: appearance B1 to B3 are in code: the tenant's `geometry.rs` calls `tissue_colour`, `--body-proportions varied|fixed` exists in `mesocosm-genet/src/main.rs`, and `isocosm/src/datasheet/native.rs` takes appendage chains (the refusal is gone). Their picture and A1 evidence was not re-run; treat them as closed.
- 2026-10-28: no `blend_radius` and no area-scaled actuation in code, so BO1 and BO2 are unbuilt. `Walking.balance` (`spatial/embodiment/walking.rs`) and inherited limb sizes (`bodied.rs` `limb_scale`) exist.
- 2026-10-28: no `BodyBindings` in this repository or in mere. `isometry-runtime` is gone; `TactileWorld` lives in `mesocosm-runtime/src/tactile.rs`; the query-refresh part of the old plan (Q15) is done. In mere, the conatus engine plan carries the "Body bindings" text (documented, not built) and the composition plan's C4 passage still reads "Open."; the `Engine` doc comment now says durable source bindings remain outside the runtime, so recheck A.2.1 at mere main before BO16.
- 2026-10-28: the old plans' superseded statements: the native adapter refuses chains (D2 lifted it); the rest-volume centre as the support proxy and "the presentation snaps an admitted turn" (posed-mass balance and supported turns replaced them for new worlds); "no admitted adjacent transfer" in the retained steep world (intermediate stances recover it); "richer turning and connected founding placement remain separate" (both landed; one search still exhausts 256 nodes); the Cambium supplier ambiguity of the senses plan appears resolved by later pins, not re-verified.
- 2026-10-28: the old body plan's receipt paths are Windows-style and unreachable on this machine.

## Progress
- 2026-10-28: cut from the body appearance, foot placement, senses and body binding plans.
