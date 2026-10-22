# Body appearance: shape suggests function, several anatomies per function

**Status (2026-10-22):** plan. Four forks answered by Mark on 2026-10-22,
recorded below in his chosen words; none numbered yet. Nothing built.

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
  cross-section (`shared/isocosm/src/systems/carry.rs:41`). Nothing scales a
  limb's actuation by area or a body's mass by volume.
- The contact gait and contact-preserving rest poses (ed3ee3bf, cd451f5c)
  plant feet from whatever support parts a body has; today those are single
  segments, not chains.

## Phases

Each phase is its own lane in its own worktree, verified there before main,
with a picture as its receipt where the change is visible (doctrine, Method
4). Any fork with more than one defensible answer returns to Mark.

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

A limb's actuation scales with its cross-section and a body's mass with its
volume, so a long slender link reaches but carries less and a stout one bears
more. The constants are Mark's; this phase proposes them with the measured
range and builds nothing until they are numbered.

Done when: the gait refuses a stance whose feet cannot bear the body and
chooses another; the same seeds still reproduce; a proposal of the constants
with evidence has gone to Mark and come back numbered.

## Open

- Whether "proportions" (broader inherited variance in segment length and
  girth) is its own phase after B5 or folds into B2.
- Whether the far rungs of the detail ladder (doctrine, Presentation 2) need
  the function tint, or only the marks.

## Progress

- 2026-10-22: plan written from the body-appearance research round and the
  four cards above; nothing built.
