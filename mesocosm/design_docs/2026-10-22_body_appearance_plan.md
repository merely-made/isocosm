# Body appearance: shape suggests function, several anatomies per function

**Status (2026-10-26):** in progress. Actual posed-matter balance is qualified;
strength limits remain open. Its receipt is three ordinary ticks; the 20/20
ecology gate below predates this opt-in. Four forks answered by Mark on 2026-10-22,
recorded below in his chosen words; none numbered yet. Materials, named
founding shapes and native chains are connected and captured. The ordinary
foot-swing gate passed in the foot-placement follow-up. B2 now connects native
recipe proportions and role-based selection, with five captured foundings and
ordinary supported walking. The native three-epoch ecology gate passes 20/20;
119 focused checks pass. Curved leaf/shell contents, B4 and B5 remain open.

## What this plan is for

Generated bodies should read as creatures whose shape suggests what they do,
while the sim keeps several anatomical solutions to one function (the
many-to-one mapping of Wainwright et al., 2005). The playtest of 2026-10-19
found bodies hard to read; the render repair of the same day fixed the
material ids and the shared "undrawn" tag; the camera moved to the wing's
dimetric on 2026-10-22 (doctrine, Presentation 1). What remains is the bodies
themselves: blocks from the generator, no tissue colour on the smooth path,
one blended chain of spheres, and limbs that exist as content but never reach
play.

Research read for this plan, with what each supplies: Wainwright et al. 2005
(many structures, one performance: the sim's side); Sims 1994 (actuation
scales with cross-section, mass with volume, so size changes movement); the
Spore liner notes and the 2008 motion-retargeting paper (readability from
authored deformable parts plus skeleton-free motion goals); Gourmel et al.
2013 (gradient-based implicit blending removes bulges and keeps sharp detail,
at shader cost); Cieslak et al. 2011 (aspect-oriented functional–structural
plant modelling: structure and functions as separately written, woven
aspects, which is the shape of our part graph plus functional networks, not
a mechanic to copy).

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

## Findings (verified 2026-10-22 against main at 9e8e73e8)

- The founding sheet composes eight bodies from legs, eyes, pads, fronds,
  crops, a shell, a rod and a blade
  (`shared/isocosm/datasheets/foundings/base.toml`). The generator still
  builds every axial segment as `block`, rods for limbs, fronds for producers
  (`shared/isocosm/src/bodied.rs:199-205`).
- Chains are declared in the jointed sheet
  (`shared/isocosm/datasheets/foundings/jointed.toml`, `[[chain.leg]]`,
  `[[chain.leaf]]`), and the native adapter refuses any entry with a chain:
  "native recipes carry no appendage chain"
  (`shared/isocosm/src/datasheet/native.rs:93`).
- The smooth body adapter supplies no materials: "Native bodies carry no
  process mosaic yet, so no materials"
  (`mesocosm/crates/mesocosm-genet/src/section/bodies.rs:65`). The tenant
  geometry path already tints tissue by phase and fractions
  (`mesocosm/crates/mesocosm-genet/src/section/tenant/geometry.rs:131`).
- The sim measures length, area, cross-section and volume
  (`shared/isocosm/src/rules/amount.rs:240`,
  `shared/isocosm/src/anatomy.rs:75`), and carrying already scales by
  cross-section (`shared/isocosm/src/systems/carry.rs:41`). Adult mass already
  scales by voxel volume (`shared/isocosm/src/anatomy.rs`, `ceiling`); limb
  actuation by area and weight-bearing stance admission remain unbuilt.
- The contact gait and contact-preserving rest poses (ed3ee3bf, cd451f5c)
  plant feet from whatever support parts a body has; today those are single
  segments, not chains.

## Phases

Each phase is verified before completion, with a picture as its receipt
where the change is visible (doctrine, Method 4). Follow the workspace rule:
work on main unless a concurrent source collision requires isolation. Any fork with more than one defensible answer returns to Mark.

### B1. Materials through the smooth path

The smooth body adapter carries the same tissue colouring the tenant path
already computes, so a played body's parts show what they do.

Done when: a `--play` frame shows parts tinted by function on smooth bodies;
the tenant and smooth paths read one colouring function, not two; the
existing render tests and the 2026-10-19 render evaluation frames hold or are
refreshed, one frame each for the default and across views.

### B2. The generator uses the sheet's shapes

Random lineages stop being blocks: the generator draws axial and borne
shapes from the founding sheet's vocabulary by the part's role (collecting
surfaces as blades, fans or fronds; storage as bulbs or thick segments;
appendages as short stout or long slender links; feeding structures as
openings, tubes or gripping branches), with inherited variance over the
choice.

Mark's follow-up on 2026-10-22 folds proportions into this phase: reuse the
existing shapes and functional traits, then capture an unchosen seed moving.
New ordinary play opts into saved native proportion choices; explicit named
presets stay fixed unless `--body-proportions varied` is requested. The
`fixed` setting retains the preceding play founding. Historical saves omit
the opt-in and retain their recipes and digests.

Done when: five seeds of the ordinary founding produce lineages whose
silhouettes differ in a contact sheet; every chosen shape is one the sheet
already declares; A1 still passes its unchanged 20/20 gate on the standard
seeds; no new shape name is coined (doctrine, Naming 3).

### B3. Native recipes accept chains

The native adapter admits the jointed sheet's chains; admission prices each
step as it prices any part; the gait plants the chain's foot, not its root.

Done when: a founding with jointed legs runs under `--play` with planted and
support contacts from feet; a refused chain step keeps the biology and
withdraws the picture, as a refused placement does today; the jointed sheet's
own tests still pass; one frame of a legged body standing.

### B4. Blending per connection

Each connection in the part graph carries a blend radius: soft at joints,
crisp at plates, shells and mouth openings. The renderer reads it; no part
changes its envelope.

Done when: a frame shows a mouth opening and a plate edge as crisp against a
soft joint on the same body; the implicit renderer's tests cover both
settings; cost measured against the current single blend on the same frame
and reported, not judged. Gradient-based blending (Gourmel) stays an option
if bulges remain, as its own later phase.

### B5. Actuation by area, mass by volume

**Actual-balance slice, 2026-10-26: implemented and qualified.** Mark authorizes wiring
posed parts and native tissue mass into support, with an unchosen moving
capture and measured loads before strength constants. `git pull --ff-only`
reports current upstream. This slice does not select strength constants.
Receipts: `Code/testing/mesocosm/posed-balance-2026-10-26/`, cap 100 MB;
reuse the existing playable target and Cargo home.

Done-conditions: read mass from living parts' native matter ledgers; reuse
the existing paid-chain skins for posed part centres; share the reading in
grounding, route admission, gait planning and validation. Mass changes must
invalidate physical cache readings. Omitted saved rules retain their former
balance law. Verify a limb movement and a changed tissue distribution alter
the support answer, replay older histories, and capture an ordinary unchosen
world with supported swings. Report mass, mass-centre shifts and supported
contact area without inventing per-foot reaction forces or strength limits.

Findings, 2026-10-26: `embodiment::support::centre_local` weights rest pivots
by box volume, rounded to whole cells. `walking::Checks` prepares that centre
once, so solved limb skins cannot move it. Isometer's chain solver already
provides the posed link endpoints and pad, and `meaning::mass` reads exact
matter from a ledger. Isocosm owns the mass reading; geometry remains in
Isometer and the existing support polygon remains the admission machinery.

Progress, 2026-10-26: new physical foundings save `Walking.balance = true`.
Isocosm reads each living part's current matter ledger; Isometer's existing
graph pivots, paid capsule midpoints and pad centres supply its placement.
Fixed-point i128 moments retain sub-cell precision. The same reading reaches
standing, neighbours, arrival admission, supported pivots, gait planning and
complete validation, including native births and runtime pose refresh. Mass
snapshots participate in route keys; changing a context clears its dependent
posture/admission/grounding readings. Older saved rules keep their old law.

409 controls pass: 289 native, eight shared-leg, 111 receiving and the explicit
cached/uncached cold-failure comparison. Twenty-five existing opt-in tests are
skipped; one older eighteen-seed play control is filtered. Thirteen older
histories and the new history replay exactly. The inspector initially treated
departed visits as occupants, reporting a false later-route collision. Its
occupancy filter now matches native current-site/current-visit semantics;
the original failure and corrected inspection remain in the receipt packet.

The unchosen clock draw starts at `1793059475430350300`; the existing search
accepts candidate 19/128, seed `12743438241358451287`. Three ordinary ticks
produce 39 verified food steps across 19 walkers, hash `99077b277fd86c1a`.
All 5,031 sampled gait phases have supported posed mass centres. The actual
24-frame motion capture has 14 supported swing frames, 20 weight shifts,
zero projection failures and no reported visible native-box intersections;
18/21 comparable same-cut pairs change pixels. Developer intents stay zero.

Measured part matter is 10 mg for every sampled walker. Bearing feet range
from three to four, summed supported pad contact cells from 7 to 48, and
aggregate mass/contact cell from 0.208 to 1.429 mg (median 0.476). Both link
cross-section readings are 9 in native anatomy units. The posed centre lies
1.587-3.632 cells horizontally from the former rest-volume proxy; straight
step articulation moves the body-relative centre by up to 1.215 cells.
These are quasi-static part-lumped readings, with pad contacts summed per
foot, not a per-foot force split. This short draw does not sample grown mass
or varied link sections and cannot calibrate strength constants. Frame timing
shares the machine with compilation; no latency comparison is claimed.

The remaining strength phase: a limb's actuation scales with its cross-section
and a body's mass with its
volume, so a long slender link reaches but carries less and a stout one bears
more. The constants are Mark's; this phase proposes them with the measured
range and builds nothing until they are numbered.

Done when: the gait refuses a stance whose feet cannot bear the body and
chooses another; the same seeds still reproduce; a proposal of the constants
with evidence has gone to Mark and come back numbered.

## Open

- Curved leaf and shell contents inside the declared volumes: the captured
  plates still read as panels even with better body proportions.
- Whether the far rungs of the detail ladder (doctrine, Presentation 2) need
  the function tint, or only the marks.

## Progress

- 2026-10-22, proportions follow-up: work on main. Required receipts are
  deterministic development/serialization checks, a five-seed silhouette
  contact sheet, actual intermediate GPU frames of one clock-drawn seed
  walking with native support, and save replay. Total retained cap: 100 MB
  under `C:/Users/mark_/Code/testing/mesocosm/body-proportions-2026-10-22`.
  Reuse the existing playable Cargo target and `isocosm-foraging` Cargo home;
  create no worktree or isolated target. Functional life history biases
  native recipe choice (capital: deeper thorax, provision-first: fuller
  collecting crown), with no capability gated by shape. Paid limb chains
  and foot-station counts remain inherited fixed structure. B4 and B5 are
  separate unfinished work.

  The old roster assigned its tier entries in order, so a one-fauna world
  always got the same browser. Its two five-segment bare backbone runs and
  long tail made the long silhouette. The base stalk also still grew along
  the old horizontal axis. The new opt-in selects existing authored plans
  with seeded life-history weights and changes their native tagma counts and
  mass kinds: `block`, `broad` and `slim`, with their existing cell allocations.
  Consumers retain paid chains and fixed leg stations. Capital breeders get
  deeper thoraxes; provision-first producers favour the branching plan and
  fuller stalk crowns. Producers draw shorter patches, compact branching
  crowns or genuinely upright stalks. These are lineage recipes used by
  development, capacity, placement, reproduction and rendering.

  The first clock draw was `1792712288221676900`; its first physically
  food-eligible candidate was number 10, `15301219037052331983`. It took two
  admitted food-directed steps with zero developer world intents. Twenty-four
  actual GPU frames include 21 supported swing frames, zero projection
  failures and zero intersections between placed bodies. Its planted feet span
  terrain heights 1117 to 1122. The slowed WebP preserves the captured pixels
  and records the original timing separately.

  The same seed and camera with `--body-proportions fixed` gives a 42 mm
  walker; the new body spans 26 mm on its longest footprint axis. Five
  independent clock draws produce walkers spanning 19 to 27 mm, with both
  four-legged plated and six-legged forms, and compact/raised producers.
  The flat shell plates still look like side panels, and upright fronds can
  read as signs. This is a captured proportion improvement. B4's connection
  blending and B5's load rules remain separate. Startup still rejects many candidates
  for physical stands or nearby food; its eligibility rules are unchanged.

  The 119 focused native/host/runtime checks pass, including deterministic
  regeneration, old serialization, inherited development, five native bodies
  standing and swinging in two orientations, and the new ordinary-walking
  regression. Replays retain `1ca7fd30b469cc59` for the new walking save,
  `0137807de2708f28` for the preceding foot-placement save, and
  `e922048bc4f566c3` for the earlier rich-body save. The explicit ecology
  check reuses A1's existing three-epoch runner, conservation assertions,
  two execution-mode replays and unchanged qualification predicate on seeds
  0 to 19. It uses the existing `ISOCOSM_REPRO_WORKERS` setting; the initial
  two-worker run was stopped after three qualified draws and retained before
  the six-worker run. All 20 draws pass the unchanged qualification predicate,
  with conserved matter on every round and exact final-save replay in both
  execution modes. This is the native site-grain ecology gate, separate from
  the actual headed physical movement above. Retained receipts total under
  10 MB of the 100 MB cap. The existing playable checkout, target and Cargo
  home remain for running and rebuilding the app; no new build locations were
  created. Final results are in the
  [receipt packet](C:/Users/mark_/Code/testing/mesocosm/body-proportions-2026-10-22/summary.json).

- 2026-10-22: plan written from the body-appearance research round and the
  four cards above; nothing built.

- 2026-10-22: the current authorized integration connects B1 and B3, with a
  selectable named founding route for existing shapes. This does not complete
  B2's inherited role-based random shape selection or its five-seed gate,
  B4's per-connection blending, or B5's load-bearing dynamics. The live
  qualification is recorded in the orthographic voxel presentation plan.

- 2026-10-22: the [foot placement follow-up](2026-10-22_foot_placement_plan.md)
  connects actual paid soles to native terrain support and the smooth path's
  shared integer chain skin. The ordinary unchosen regression now records
  supported foot swing, closing the earlier zero-swing qualification gap.
  The support polygon uses the document's volume centre as a geometric proxy;
  this does not complete B5's actuation/load rules or its constants.
