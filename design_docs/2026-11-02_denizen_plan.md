# Denizens: bodies you play, looks you see, companions you direct, eyes you look through

**Status:** 2026-11-02: plan. Nothing built. Its ten questions are answered
as cases 1133 to 1144 (Decisions); P1 to P3 are ready to start. P6 carries
pieces of case 1134 still to rule, and P7 waits on the creature-minds seam.

**For:** carrying out cases 1128 to 1131, ruled by Mark on 2026-11-02:

- **1128**: Cosmograph gains generated body plans for sophonts and denizens,
  drawn per lineage within bounds like other recipes; a creator edits them
  within those bounds; all three games read the same bodies (doctrine World
  10, Bodies 1 and 11, Eponym 4).
- **1129**: Eponym draws bodies through the layered look of VTT 7 and
  isometer's baked voxels, as Isocosm's tokens do.
- **1130**: a player gives companions orders (go, hold, fight, carry) that
  they weigh with their own minds and may refuse; where a companion's
  agreement allows, the player may drive it directly; the player still lives
  one creature (Eponym 1).
- **1131**: the shared renderer gains a perspective camera for first person
  and over the shoulder in every product, switchable live against the
  dimetric overview (Presentation 1).

Mark's words, which started it: "eponym... i gotta say, you shouldn't be
blocks, i was more picturing, like, be an individual...? imagine the old
baked voxel tokens in isocosm. like, that's at least approaching a character
model. although i would expect that it would be, y'know, a unique, generated,
designable humanoid/denizen that you'd be playing, among other procedurally
generated, unique entities... seems pretty far from what we've got. although
the perspective isn't bad!" Then: "though first person should also be
possible in addition to the kenshi-esque overview. being able to control
your character and direct others..."

**Not here:** creature minds and the chooser's learned policy (the creature
minds brief, `mesocosm/design_docs/2026-10-31_creature_minds_brief.md`, and
the sim execution plan); bodied plants and the food web (cases 1114 and
1115); Mesocosm's 2.5D foraging, spatial and speed work; Eponym's round,
debug injury and acceptance failures (the foreground lane, the Eponym
overlay plan); joint ranges and the swing cap (case 1088, Bodies 16); the
board's remaining drawing (`2026-10-28_board_on_isometer_plan.md`).

**Receipts:** `archive_docs/receipts/2026-11-02_denizen_plan_receipts.md`,
created with the first landing (DOC_POLICY §8).

## Constraints

- **Owning layer first** (`CLAUDE.md`, "Working here"; Stack 3 and 4). The
  sim owns bodies, bounds, minds, agreements and who is played; isometer owns
  geometry, voxels, looks' shape and the camera (Bodies 2, Stack 5);
  isomere owns wing GUI panels, promoted from an existing product's, never
  written fresh, and never names a voxel (`shared/isomere/src/lib.rs`); a
  product owns presets, keys and its own panels' wording.
- **No second renderer and no second simulation** for a camera or a person
  shift (founding record §1, "The cost to book"; vessel briefs, "First person
  as a setting").
- **Done-conditions are draws from a declared space, never fixtures**
  (case 15). Old worlds keep their digests: new founding data is opt-in, as
  `Bodies::proportions` and `Bodies::sensing` are (`shared/cosmograph/src/bodied.rs`).
- **Refuse, never invent** (Method 8): an edit or order outside what is
  allowed is refused whole, with its reason.
- **Surfaces derive from knowledge and control** (Directing 1): first person
  shows what the played creature knows, never more.
- **Source files stay under 600 lines** (Method 3); `bodied.rs` is 475 and
  `shared/isometer/src/camera.rs` 372, so new work lands in siblings.
- **Lanes stop at forks.** Each phase's lane carries the answers it executes
  and comes back with any choice that has more than one defensible answer.
  Nothing reaches main until its tests pass in its own worktree, with
  captures where the change is visible.
- **Concurrent work, read and never edited here:** the foreground lane
  (Eponym's `round.rs`, the debug injury, acceptance failures); the food-web
  lane (`ecology/world_time.rs`); the Mesocosm 2.5D session (`spatial/`,
  `foraging`, `speed.rs`); the creature-minds session (minds,
  `directing/choice.rs` and `policy.rs`, `lineage/boundary.rs`). A phase
  needing a seam one of them owns (P7, the chooser) waits.

## Phases

Order, and why:

1. **P1, camera:** case 1131's title says "comes first"; only isometer and
   presets, no lane in its way, visible in all three games at once.
2. **P2, body plans:** the root of case 1128; everything visible after it
   reads these bodies.
3. **P3, look:** needs only a part tree, so it runs on today's bodies, and
   Eponym then consumes a shared look rather than building one.
4. **P4, Eponym's look:** proves case 1129 on the six-part body, so the look
   change and the body change are seen separately.
5. **P5, a played denizen:** needs P2 and P4; where Mark's picture is met.
6. **P6, creator:** needs P2's templates and founding count, P3's look,
   and a played denizen (P5).
7. **P7, orders:** waits on the creature-minds session's chooser; after the
   bodies, to keep one thing moving at a time.
8. **P8, direct control:** needs P7's agreements and P1's camera.

### P1. The shared perspective camera

**Owning layer:** isometer (`shared/isometer`), with a preset and a key per
product.

**Touches:**
- `shared/isometer/src/camera.rs` and siblings (`camera/`): `SlabCamera`
  derives the tracer camera (`trace`), the cull window (`window`), the raster
  matrix (`clip_from_world`), the body clip (`clip`) and picking (`ndc_of`,
  `pixel_of`), all orthographic now. It becomes one camera with two
  projections, slab and perspective, each derived quantity answering for
  both; dimetric, first person and over the shoulder are its constructors,
  and any rename goes to a naming round (case 1140).
- `shared/isometer/crates/isometer-lens/src/tracer/types.rs`: already has
  `TraceCamera::perspective` (the DDA shares one path for both projections);
  only its use from the scene camera is new.
- `shared/isometer/crates/isometer-render/src/live_body*`: the body raster
  takes a perspective `clip_from_world`.
- `shared/isometer/src/scene/residency/framing.rs`, `scene/coordinates.rs`,
  `scene/columns.rs`, `query.rs`, `glyphs.rs`, `bodies.rs`: framing, picking
  and culling under a frustum.
- Presets and a live toggle: Eponym's `producer/policy.rs` (`CameraPolicy`)
  and `producer/handle.rs`; Mesocosm's `mesocosm-genet/src/section/camera.rs`
  (`CameraMode`); Isocosm's `isocosm-views/src/scene/board.rs` and
  `scene/world.rs`. First person's eye sits at the part with the most sense
  cells, else the top of the root (case 1143).
- Eponym's app-local perspective (`eponym-client/src/scene.rs`'s room camera,
  `crossing.rs`, `gpu/body.rs`, `bin/room.rs`, `residency.rs`'s planning
  camera) is surveyed. Its retirement, or why it stays, comes back at this
  phase's checkpoint as a question.

**Done when:**
- In isometer's camera tests, under perspective as under the slab, a voxel's
  traced hit and its rastered pixel coincide within one pixel. A body
  just inside the frustum is drawn and one just outside is culled, with both
  controls in the same test. Picking returns the body under the cursor.
- Residency under perspective keeps the visible ground within the atlas
  budget and reports what it omits (Presentation 5).
- In each product, one key switches between the dimetric overview, over the
  shoulder and first person on the played body, live. The switch writes no
  sim state and rebuilds no scene, and the overview the switch returns to is
  the one left.
- The eye stands at the most sense-celled part for the six-part body, a
  roster fauna and a body with no sense part (the root's top), in one test.
- Existing camera tests and each product's dimetric captures are unchanged.
- **Headed:** for each product, the same moment in the three views, in
  `Code/testing/<product>/denizen/camera-*.png`. Every frame is examined
  whole, not only the body (terrain seams, culled edges, the HUD).

**Needs:** cases 1140, 1143. Nothing from other phases.

### P2. Denizen body plans in the sim

**Owning layer:** the sim (`shared/cosmograph`).

**What:** the denizen family of case 1133, an upright trunk, a head and
paired jointed limb chains, with counts and proportions drawn per lineage
within bounds a world may widen; a humanoid is one region of that space.
Its plans are written as data a template can name, because P6's voxel
editor snaps to editable body-plan templates (case 1134). A lineage's
rarity is its founding count (case 1138).

**Touches:**
- A sibling of `shared/cosmograph/src/bodied.rs` (`bodied/denizen.rs`) for
  the denizen draw. `drawn` today makes stretches of 1 to 6 `block` segments
  bearing rods or an eye; it never uses the grammar's `parent`/`anchor`
  layouts or jointed `chain`s.
- `bodied.rs`'s `Bodies` (the bounds, `check`) and `embody` (founding by
  kingdom, World 8, and each lineage's founding count);
  `bodied/proportions.rs` (limb scale is the only proportion drawn now,
  `limb_scale` 1 to 2).
- `shared/cosmograph/datasheets/foundings/`: a palette, chains and named
  plan templates for the denizen family. `jointed.toml` already has upper
  leg, lower leg and foot links (`chain.leg`) and `branching.toml` layouts
  with `parent` and `anchor`; `datasheet/schema.rs` only for new words.
- Read, not edited: `rules/recipe.rs` (`Tagma`, `Recipe`, `ChainLink`,
  `Policy`), `development.rs`, `speed.rs` (Bodies 16 reads leg length),
  `lineage/` (the epoch boundary revises recipes).

**Done when:**
- Over a declared space of seeds and bounds, every drawn denizen plan passes
  `Bodies::check`, develops without refusal, carries its reproduce cell
  (case 547), senses under `Bodies::sensing` (Bodies 15) and, where legged,
  reads a speed from its leg (Bodies 16). Siblings differ by soma.
- Across draws, each drawn proportion fills its declared range, with the
  distribution in the receipts. A bound outside the generator's domain is
  refused (negative control).
- A world that does not opt in keeps its recipe lottery and digest, checked
  against a saved founding.
- The plan is a body, not a declared sophont: sapience stays measured
  (Directing 3), and no denizen draw sets it.
- Each named template develops as a recipe by itself, and a draw from it
  stays within the family's bounds.
- A lineage founds with the count its rarity sets. Drawn lineages keep
  World 8's floor of six (case 1090); a count below the floor is refused here
  and admitted only for a player's designed lineage in P6 (case 1138).
- **Headed:** a lineup of drawn denizens baked through isometer-mesh's
  `flatten` and sprite bake, in `Code/testing/wing/denizen/lineup.png`, beside
  the same seeds' fauna as the control.

**Needs:** cases 1133, 1138. Nothing from other phases.

### P3. The layered look from the part tree

**Owning layer:** isometer-mesh for the derivation (it owns `Appearance`,
`compose`, `Palette`, `flatten` and `BodyProfile::voxels`); the layered
resolution moves there for every product (case 1136). Product packs stay
product data.

**Touches:**
- `shared/isometer/crates/isometer-mesh/src/bake/recipe.rs`, `flatten.rs`,
  `bake/body.rs`, and a new sibling for the derivation by layer (shape from
  the part tree, palette from material, finish from make, style from a pack,
  wear from condition, variation from the seed, overrides last: VTT 7).
- `crates/isocosm-views/src/appearance.rs` (`Library::resolve`),
  `appearance/items.rs`, `scene/tokens.rs`: these resolve today from a pack
  class's `.vox` slot layers (`hood.vox` in the core pack), never from a part
  tree. They come to read the shared derivation. A sim-backed token's shape
  is its part tree, its class's layers styling it; sim-off tables keep class
  shapes (case 1141).
- `shared/isometer/src/bodies.rs` (`SceneVolumes`): a voxel volume per part
  up close, one baked volume for the far sprite rung (case 1139).
- Microvoxels (case 1149): the look draws each part at a number of
  microvoxels per sim cell, shaped by its form and the layered look; the sim
  keeps cell grain. P6's voxel editor works at the same grain. Finer body
  cells than terrain's come later, where play needs them, and are not this
  phase.

**Done when:**
- One derivation turns a body document and its lineage into a look, and a
  sim-backed Isocosm token resolves through it, shape from its parts.
- Every existing class token resolves to the same key and voxels as before
  (regression over the core pack), and the preview and the board still agree
  (the existing appearance tests).
- A part's voxels stay within its declared box, and the part ids a wound
  addresses are the ones drawn.
- A part's microvoxels stay within its cells, and a severed part's cells
  vanish from the drawn look.
- **Headed:** the Isocosm board with a sim-backed token drawn from its part
  tree beside a class token, scene and DOM boards, in
  `Code/testing/isocosm/denizen/`.

**Needs:** cases 1136, 1139, 1141, 1149. Bodies from P2 are not needed: it runs
on the roster and the six-part fixture.

### P4. Eponym draws through the look

**Owning layer:** Eponym (`eponym-client`), consuming P3.

**Touches:**
- `eponym/crates/eponym-client/Cargo.toml`: isometer-mesh, directly or
  through `isometer`'s re-export. No Eponym crate depends on it now.
- `producer/source.rs`: `drawable` and `frame` pass bodies as
  `SceneVolumes::DeclaredSolid` part boxes, tinted played or other
  (`SceneModel::tint`). They come to pass the derived look. `scene_pose` keeps
  standing the body on its feet.
- `producer/policy.rs` (`Appearance`'s `played` and `other` tints) and
  `producer/handle.rs`. The played body stays identifiable (it is
  `always_visible` and followed), by a mark rather than a tint.
- Not `round.rs`: `drawable` reads `Round::drawn` and leaves it as it is.

**Done when:**
- The played and other bodies are drawn from the derived look, and the
  producer passes no `DeclaredSolid` volume (asserted in the producer
  tests).
- A severing wound removes that part's voxels at the next frame. The
  debug injury of case 1127, which names its part, is the positive control.
- Poses still move parts, and an anatomy revision rebuilds only that body.
- **Headed:** Eponym's played world before and after, dimetric and over the
  shoulder, in `Code/testing/eponym/denizen/look-*.png`.

**Needs:** P3; P1 for the over-the-shoulder capture.

### P5. Eponym plays a denizen

**Owning layer:** the sim draws the body (P2). Eponym's founding chooses it
for a life.

**Touches:**
- `eponym/crates/eponym-play/src/founding/body.rs`: the compact six-part
  recipe (root with intake and store, two bilateral limb pairs with
  contract, support and grip, one sensor above). Also `founding/fixture.rs`
  and `founding.rs`.
- `eponym-play/src/contract/mode/life.rs`: a first life's pick (Eponym 1: a
  newly generated outsider, a birth into a lineage, or a named denizen).
- Read: `shared/cosmograph/src/arrival.rs` (an outsider is native arrival,
  case 238).

**Done when:**
- A newly generated outsider's body is a drawn denizen plan of one of the
  world's lineages, among other drawn denizens. The six-part recipe stays
  only as a test fixture.
- A save restores the same body by version tag (Eponym 4), and strikes,
  wounds and severing work on its parts (the harm tests rerun on it).
- `eponym-motion` stands and walks it at its Froude speed (Bodies 16).
- **Headed:** the Eponym client with the played denizen among others, in
  dimetric and first person, in `Code/testing/eponym/denizen/played-*.png`.

**Needs:** P2, P4. At its start, the foreground lane's current files are
checked, since both work in Eponym.

### P6. The creator: an individual that seeds a lineage

**Owning layer:** the sim admits a designed lineage and founds it; isometer
holds the voxel edit and draws the preview; isomere hosts the panels,
promoted from Isocosm's look editor. It opens only for lives not born in
play: Eponym's generated outsider, a quick-start world's authored founder, a
VTT character at creation. A life born in play stays drawn, with look
overrides only (case 1135).

**What (cases 1134, 1137, 1138):** the designer shapes one individual in a
voxel editor that snaps to P2's editable body-plan templates, and marks each
trait as the lineage's or the individual's own. Rounds of drawn variants are
then kept or discarded as fitting the lineage; the kept examples set its
distribution, and a lone example is a blueprint with minor variance (palette
swaps, for one). Rarity sets the founding count, from one, the last of its
kind, to a common share; a player's lineage may found below six, and a lone
member may find no mate, the sim deciding what follows. The rounds and the
editor are built together.

**Touches:**
- The sim: a sibling under `shared/cosmograph/src/bodied/` that turns a
  designed individual, its trait marks and its kept examples into a lineage
  recipe and bounds, admitted or refused whole against the world's `Bodies`
  bounds; its founding at its count (`embody`, or `arrival.rs` in a running
  world); `cosmograph-overlay` for the intent that submits it.
- isometer-mesh: the voxel edit on a part tree, snapping to templates, each
  edit within its part (Bodies 2: physiology never writes geometry).
- isomere: the creator panels promoted from
  `crates/isocosm-views/src/look.rs` and `appearance/creator.rs`
  (`LookDraft`, the live preview leaf): the editor, trait marks, variant
  rounds and rarity beside class, slots and palette.
- Eponym's first-life pick (P5's `life.rs`) and Isocosm's character creation.

**Done when:**
- Over drawn members of a designed lineage, traits marked the lineage's
  appear in all of them and traits marked the individual's own in it alone.
- Every member drawn after a round lies within the distribution its kept
  examples set; a lone example's members differ only by the blueprint's
  minor variance.
- A voxel edit snaps to a template, and the edited individual develops to
  the body the preview showed (the same body document digest); a save
  restores it.
- A design outside the world's bounds is refused whole by the sim, with its
  reason (negative control).
- Rarity founds the lineage at its count: a player's lineage of one founds
  and is played, and a drawn lineage of one is still refused (control).
- Isocosm's look editor behaves as before after promotion (its tests and its
  headed pair in `Code/testing/isocosm/appearance/`).
- **Headed:** the editor snapping to a template, a round with kept and
  discarded variants, and the founded lineage's lineup, in Eponym and
  Isocosm, in `Code/testing/<product>/denizen/creator-*.png`.

**Open, returned as questions at this phase's checkpoints** (case 1134 is
working direction with its pieces still to rule): how a designed lineage
enters a running world (founders arriving as outsiders, or only at a
founding); how kept examples become a distribution (their span, or a fitted
spread); what minor variance a blueprint allows beyond palette swaps; and
how a voxel edit maps back to the grammar (within a part's box, or reshaping
it). Mesocosm critters made playable in Eponym wait until the two meet on the
sim backend (case 1134) and are not here.

**Needs:** cases 1134, 1135, 1137, 1138; P2's templates and founding count,
P3, P5.

### P7. Orders

**Owning layer:** the sim's social layer and goals; the overlay contract;
Eponym's input and panel.

**Touches:**
- `shared/cosmograph/src/social/terms.rs` (`Work`: a craft key, grade and
  danger, with no target) and `social/asks.rs`. `weigh_offer` and
  `exercise` already weigh an ask by `willing.rs`'s three gates: craft,
  trust for the danger, and danger within what the peer would bear for the
  asker (ruling 60).
- `shared/cosmograph/src/goals/` (`Directive::Adopt`): an accepted order
  becomes a goal the companion pursues when its own choice allows, the
  pattern Directing 4 gives approved journeys.
- `shared/cosmograph-overlay/src/eponym/ask.rs` (`Proposal`, `WorkKey`,
  `AgreementHandle`): go and carry need a target, which `Work` lacks; its
  shape comes back at this phase's checkpoint.
- `eponym/crates/eponym-play/src/contract.rs` (`EponymIntent::Ask` routes to
  `Social::Consider` or `Social::Exercise` now); the client's order input and
  a panel showing answers.
- Not `directing/choice.rs` or `policy.rs`: the chooser taking up a goal is
  the creature-minds session's seam, and this phase waits for it.

**Done when:**
- Each of go, hold, fight and carry, asked of a companion, is weighed. An
  accepted order is followed by the companion's own chooser. A refusal comes
  back with its premises and is shown.
- Over drawn standings, acceptance does not fall as standing rises, and the
  same order is refused at low standing and accepted at high.
- No order moves a companion by any path but its own choice: the tests
  assert no `PLAYS` or nudge from the participant to it (case 152).
- **Headed:** Eponym's panel with an order given, one accepted and one
  refused with its reason, in `Code/testing/eponym/denizen/orders-*.png`.

**Needs:** case 1142; the creature-minds session's chooser seam.

### P8. Direct control

**Owning layer:** the sim (who is played, `PLAYS`; the agreement that allows
it); the overlay contract (one actuated body); Eponym (the switch and its
camera).

**Touches:**
- `shared/cosmograph/src/directing/mod.rs` (`PLAYS`, `plays`, `played`; a
  participant may already play several bodies, cases 1062 and 1065) and
  `social/companions.rs` (`CompanionRule::Agreement`).
- `shared/cosmograph-overlay/src/eponym/actuation.rs`: "Never a second body
  (ruling 152)"; it actuates whichever one body is held.
- `eponym-play/src/contract/mode/binding.rs` and `life.rs`, kept apart from
  succession; the client's switch, with the camera following the driven body
  (P1).

**Done when:**
- Taking control without an agreement that allows it is refused, and taking
  it with one records an event (the Eponym overlay plan's "control changes
  through death or a recorded world event").
- The participant actuates exactly one body throughout. The left creature
  acts by its own mind meanwhile, and the companion can end the control, after
  which control returns.
- **Headed:** the switch in first person, through the companion and back,
  in `Code/testing/eponym/denizen/drive-*.png`.

**Needs:** case 1144; P7, P1.

## Decisions

The plan's ten questions, answered by Mark on 2026-11-02 and recorded as
cases in the design record's §0, which holds his words and each case's
links. Here, the question each answers, the reading, and where it lands.

- **Q1, 1133:** a denizen is a drawn family in the recipe grammar: an
  upright trunk, a head and paired jointed limb chains, counts and
  proportions per lineage within bounds a world may widen. P2.
- **Q2, 1134:** reframed in Mark's words. The creator designs an individual
  that seeds a lineage: traits marked the lineage's or its own, kept rounds
  of variants set the lineage's distribution, a lone example is a blueprint
  with minor variance, a voxel editor snaps to editable body-plan
  templates, and rarity runs from one to common. Working direction, its
  pieces still to rule. P2, P6.
- **1137:** the variant rounds and the voxel editor are built together. P6.
- **1138:** rarity is the founding count; a player's lineage may found below
  the floor of six. P2, P6.
- **Q3, 1135:** the creator opens for lives not born in play (outsiders,
  quick-start founders, VTT characters at creation); births stay drawn,
  with look overrides only. P6.
- **Q4, 1136:** the layered look resolution moves into isometer-mesh, for
  every product. P3.
- **Q5, 1139:** a body's voxels are held per part up close and baked as one
  volume for the far rung. P3, P4.
- **Q6, 1141:** a sim-backed token's shape is its part tree, its class's
  layers styling it; sim-off tables keep class shapes. P3.
- **Q7, 1142:** orders are in-world asks under a companion agreement,
  weighed by the existing willingness code; an accepted one becomes the
  companion's own goal. "Configure, don't command" is refined, not dropped.
  Directing 1 refined. P7.
- **Q8, 1140:** one isometer camera with two projections; presets are
  constructors, and any rename goes to a naming round. P1.
- **Q9, 1143:** first person's eye sits at the most sense-celled part, else
  the top of the root. P1.
- **Q10, 1144:** driving a companion is agreed, recorded and one body at a
  time, and the companion may end it. Directing 1 and case 187 refined;
  Eponym's `CLAUDE.md` guardrails rewritten to match. P8.

Further answers are recorded here as `denizen D<n>`, in answer order.

## Findings

2026-11-02, verified against the tree at `a7bfe12e`:

- Eponym draws each body's declared part boxes as
  `SceneVolumes::DeclaredSolid`, tinted `appearance.played` or
  `appearance.other` (`eponym-client/src/producer/source.rs`: `drawable`,
  `scene_pose`, `SceneModel::tint`, `frame`). Eponym crates depend on
  `isometer`, `isometer-core` and `isometer-space`; none on `isometer-mesh`.
- The played body is the compact six-part recipe of cases 816 and 818
  (`eponym-play/src/founding/body.rs`): a 4×4×4 root (intake 22, store 5),
  two bilateral pairs of 4×1×1 limbs (contract, support, grip), one 1×1×1
  sensor above.
- `bodied::drawn` sets no `parent`, `anchor` or `chain`; no humanoid or
  denizen plan exists. The grammar branches (`Tagma::parent`, `anchor`,
  `facing`, `socket` in `rules/recipe.rs`) and grows `ChainLink` chains, as
  `datasheets/foundings/jointed.toml` (`chain.leg`: leg, upright_leg, foot)
  and `branching.toml` layouts do.
- Isocosm's looks resolve from a pack's class bindings and `.vox` slot
  layers (`isocosm-views/src/appearance.rs`, `Library::resolve`; core pack
  `hood.vox`), and tokens are one rigid volume (`scene/tokens.rs`); no look
  reads a part tree. isometer-mesh turns a body document into voxels
  (`flatten`; `BodyProfile::voxels`, `voxels_by_part` in `bake/body.rs`), and
  the shared scene draws `SceneVolumes::Voxels` or `DeclaredSolid`
  (`shared/isometer/src/bodies.rs`).
- isometer's scene camera is orthographic only (`SlabCamera` in
  `shared/isometer/src/camera.rs`; `isometer-render`'s `Camera`).
  `TraceCamera::perspective` (`isometer-lens/src/tracer/types.rs`) is used
  only by Eponym's app-local room renderer (`eponym-client/src/scene.rs`)
  and planning camera (`residency.rs`). Every product builds its camera from
  `SlabCamera` (P1's files).
- The sim weighs asks and agreement exercises (`social/asks.rs`:
  `weigh_offer`, `exercise`) through `willing.rs`'s craft, trust and danger
  gates, and Eponym's contract routes `EponymIntent::Ask` there
  (`eponym-play/src/contract.rs`); `Work` carries no target.
- Records still disagreeing with the rulings, not edited by this plan:
  - first person in every product (1131): the vessel briefs
    (`mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md`,
    "First person as a setting, Paredros only") and the founding record's
    closing note to its camera item (lines 1220 to 1226);
  - orders as asks (1142): the founding record's "Configure, don't command"
    (§1), which the case refines;
  - a player's lineage below six (1138): World 8's "Every founded lineage
    starts with at least six members".

2026-11-02, after cases 1133 to 1144: Directing 1 refined for 1142 and 1144;
Eponym 1 already said what they need and is unchanged; Eponym's `CLAUDE.md`
guardrails were rewritten with case 1144.

2026-11-02, P1: Eponym's r1-proof residency errors over budget where the
shared residency drops and reports, which capped Eponym's perspective far
plane at 28 cells (case 1152 retires it). Mesocosm bodies stand about 24
cells against Eponym's 2, so `PerspectivePolicy::for_body` scales near, far
and shoulder by body height. `View`, `PerspectivePolicy` and `below_the_eye`
live in isometer for all three products. Smooth bodies had no perspective
path; the implicit body shader now divides by w.

## Progress

- 2026-11-02: plan written from cases 1128 to 1131; nothing landed.
- 2026-11-02: cases 1133 to 1144 folded in as Decisions; P2 and P6 reshaped for
  1134, 1137 and 1138; Directing 1 refined.
- 2026-11-02: P1 isometer and Eponym landed (`60362a2a`, `be977786`,
  `b743cf75`; cases 1150, 1151); Mesocosm's three views on Y (`381cbb0b` to
  `f51e75c1`). Open in P1: Isocosm's views (cases 1157, 1158), retiring
  Eponym's proof cameras (1152), and Mesocosm's view key (Mark asked about
  Q/E for turning, which are bound now).
