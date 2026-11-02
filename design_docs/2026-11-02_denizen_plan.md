# Denizens: bodies you play, looks you see, companions you direct, eyes you look through

**Status:** 2026-11-02: plan. Nothing built. Every phase waits on at least
one question below; P1 waits only on Q8 and Q9.

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
6. **P6, creator:** needs P2's bounds and P3's look, and a denizen to edit.
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
  `pixel_of`), all orthographic now. Each gains a perspective answer, in the
  shape Q8 rules.
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
  `scene/world.rs`. The eye's place follows Q9.
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
- Existing camera tests and each product's dimetric captures are unchanged.
- **Headed:** for each product, the same moment in the three views, in
  `Code/testing/<product>/denizen/camera-*.png`. Every frame is examined
  whole, not only the body (terrain seams, culled edges, the HUD).

**Needs:** Q8, Q9. Nothing from other phases.

### P2. Denizen body plans in the sim

**Owning layer:** the sim (`shared/cosmograph`).

**Touches:**
- A sibling of `shared/cosmograph/src/bodied.rs` (`bodied/denizen.rs` or as
  Q1 shapes it) for the denizen draw. `drawn` today makes stretches of 1 to
  6 `block` segments bearing rods or an eye; it never uses the grammar's
  `parent`/`anchor` layouts or jointed `chain`s.
- `bodied.rs`'s `Bodies` (the bounds, `check`) and `embody` (founding by
  kingdom, World 8); `bodied/proportions.rs` (limb scale is the only
  proportion drawn now, `limb_scale` 1 to 2).
- `shared/cosmograph/datasheets/foundings/`: a palette and chains for the
  denizen family. `jointed.toml` already has upper leg, lower leg and foot
  links (`chain.leg`) and `branching.toml` layouts with `parent` and
  `anchor`; `datasheet/schema.rs` only if Q1 needs new words.
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
- **Headed:** a lineup of drawn denizens baked through isometer-mesh's
  `flatten` and sprite bake, in `Code/testing/wing/denizen/lineup.png`, beside
  the same seeds' fauna as the control.

**Needs:** Q1. Nothing from other phases.

### P3. The layered look from the part tree

**Owning layer:** isometer-mesh for the derivation (it owns `Appearance`,
`compose`, `Palette`, `flatten` and `BodyProfile::voxels`), unless Q4 rules
otherwise; product packs stay product data.

**Touches:**
- `shared/isometer/crates/isometer-mesh/src/bake/recipe.rs`, `flatten.rs`,
  `bake/body.rs`, and a new sibling for the derivation by layer (shape from
  the part tree, palette from material, finish from make, style from a pack,
  wear from condition, variation from the seed, overrides last: VTT 7).
- `crates/isocosm-views/src/appearance.rs` (`Library::resolve`),
  `appearance/items.rs`, `scene/tokens.rs`: these resolve today from a pack
  class's `.vox` slot layers (`hood.vox` in the core pack), never from a part
  tree. They come to read the shared derivation.
- `shared/isometer/src/bodies.rs` (`SceneVolumes`): per-part voxel volumes,
  if Q5 rules per part.

**Done when:**
- One derivation turns a body document and its lineage into a look, and a
  sim-backed Isocosm token resolves through it, shape from its parts.
- Every existing class token resolves to the same key and voxels as before
  (regression over the core pack), and the preview and the board still agree
  (the existing appearance tests).
- A part's voxels stay within its declared box, and the part ids a wound
  addresses are the ones drawn.
- **Headed:** the Isocosm board with a sim-backed token drawn from its part
  tree beside a class token, scene and DOM boards, in
  `Code/testing/isocosm/denizen/`.

**Needs:** Q4, Q5, Q6. Bodies from P2 are not needed: it runs on the roster
and the six-part fixture.

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

### P6. The creator

**Owning layer:** the sim admits a designed body within bounds. isometer
draws the preview. isomere hosts the panel, promoted from Isocosm's look
editor. Each product names when it opens.

**Touches:**
- The sim: a sibling under `shared/cosmograph/src/bodied/` that admits an
  explicit draw (the chosen values a designed body was developed from)
  against its lineage's bounds, and `cosmograph-overlay` for the intent that
  submits it.
- isomere: the creator panel promoted from
  `crates/isocosm-views/src/look.rs` and `appearance/creator.rs`
  (`LookDraft`, the live preview leaf), with body rows beside class, slots and
  palette as Q2 rules.
- Eponym's first-life pick (P5's `life.rs`) and Isocosm's character creation,
  as Q3 rules.

**Done when:**
- Every value the creator offers lies within the lineage's bounds. A crafted
  submission outside them is refused whole by the sim, with its reason
  (negative control).
- An accepted design develops to the body the preview showed (the same body
  document digest), and a save restores it.
- Isocosm's look editor behaves as before after promotion (its tests and its
  headed pair in `Code/testing/isocosm/appearance/`).
- **Headed:** the creator in Eponym and in Isocosm, before and after one
  edit, with its preview, in `Code/testing/<product>/denizen/creator-*.png`.

**Needs:** Q2, Q3; P2, P3, P5.

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
  `AgreementHandle`): a target for go and carry if Q7 rules one is needed.
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

**Needs:** Q7; the creature-minds session's chooser seam.

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

**Needs:** Q10; P7, P1.

## Questions for Mark

Each names the doctrine it bears on. Recommendations come first. None is
decided here. Answers are recorded as `denizen D<n>` in the Decisions
section, or as cases where code or another plan comes to rely on them.

### Q1. What a denizen body plan is made of (Bodies 6, World 8; case 1128)

Evidence: the recipe grammar already branches (`Tagma::parent`, `anchor`,
`facing`) and grows jointed chains (`ChainLink`; `jointed.toml`'s upper leg,
lower leg and foot). Authored roster bodies use both, while the lottery
(`bodied::drawn`) uses neither: it makes 1 to 4 stretches of 1 to 6 block
segments under a mouthed head. The only drawn proportion is `limb_scale`, 1
to 2. Sapience is measured, never declared (Directing 3), and in deep time
people grow out of lineages (World 10).

1. **(Recommended) A denizen family in the existing grammar.** A third
   family beside flora and fauna: an upright trunk tagma, a head tagma on
   it, and paired jointed limb chains, with the trunk's segments, the limb
   pairs, links per chain, link proportions and head share each drawn per
   lineage within `Bodies` bounds a world may widen. "Humanoid" is one region
   of that space. No new vocabulary, and the plan is a body, not a
   sophont.
2. **Authored archetypes with drawn proportions.** A datasheet of plans
   (biped, four-legged with hands, centauroid, armed serpent); each lineage
   picks one and draws its proportions within ranges. More legible and
   easier to bound, less generative.
3. **No separate family.** Widen the fauna lottery to draw layouts and
   jointed chains, bias limbed, sensing consumers toward grip, and play
   whatever passes the sapience measure, with authored founders in quick-start
   worlds. Most faithful to World 10, but a given world may never grow a
   playable body, and the first is late.

### Q2. What the creator edits, and how an edit stays within bounds (VTT 7, Eponym 4; case 1128)

Evidence: Isocosm's creator (`look.rs`, `appearance/creator.rs`) edits a
token's class, slot layers and palette with a live preview, and the token
keeps only its overrides (`TokenAppearance`). A body is developed from a
recipe and a seed (`development::soma`), and restore regenerates a
founding by version tag (Eponym 4).

1. **(Recommended) The individual, within its lineage's bounds.** The
   individual's proportions, link lengths and counts are edited where the
   recipe's variance allows, along with the look's layers above shape. The
   rows offer only in-bound values. The result is an explicit draw (seed plus
   chosen values) that the sim admits or refuses whole. Lineage recipes stay
   the world editor's, later.
2. **The seed only.** Reroll within bounds, plus look overrides; always in
   bounds by construction, but "designable" is thin.
3. **Individual and lineage by seat.** A player edits their own individual,
   and a creative observer or world author edits the lineage's recipe within
   the world's bounds, in one creator.

### Q3. When the creator opens (Eponym 1, VTT 2)

Evidence: Eponym 1's first life is a newly generated outsider, a birth into
a lineage, or a named denizen of the drawn world. VTT 2: the seat
controlling a body drafts its character. Every birth is a transfer drawn
from the parents (Bodies 8).

1. **(Recommended) Only for lives not born in play.** A newly generated
   outsider, an authored founder in a quick-start world, and a VTT character
   at creation. Births and named denizens are drawn and only their look
   overrides are editable.
2. **Every first life.** A birth into a lineage is also edited within the
   lineage's bounds, as a designed child.
3. **Any time, by look only.** Body choices only at creation, and look
   overrides (palette, style) whenever the seat chooses.

### Q4. Where the shared look lives (VTT 7, Stack 3; case 1129)

Evidence: isometer-mesh already owns `Appearance`, `compose`, `Palette`,
`flatten` and the sprite bake. The layered resolution (`Library::resolve`,
the `appearance.toml` pack format, swatches, item layers) lives in
`isocosm-views`, a Cambium view crate. isomere never names a voxel.

1. **(Recommended) isometer-mesh.** The derivation and the pack binding
   format move to isometer-mesh's bake, and products load their own packs.
2. **A new shared crate under `shared/`.** It holds looks apart from
   meshing, and needs a naming round (Naming 3).
3. **Eponym depends on `isocosm-views`' appearance module.** Least moved,
   but one product's view crate becomes another's dependency.

### Q5. A body's voxels per part, or one volume (Bodies 2, Bodies 12; case 1129)

Evidence: Isocosm's tokens are one composed volume
(`TokenBody::from_voxels`), rigid and turned by facing. Eponym's bodies pose
and sever by part, and the scene draws them per part from declared extents.
`flatten` documents the split: the live renderer wants a mesh per part so
limbs move, and the sprite baker wants one grid.

1. **(Recommended) Per part near, one baked volume far.** Each part gets its
   own voxel volume, so limbs move and severing shows. The whole-body bake
   serves the sprite rung (Presentation 2).
2. **One volume everywhere, as tokens are.** The cheapest, matching today's
   tokens, but limbs neither move nor visibly sever.

### Q6. A sim-backed token's shape: part tree or pack class (VTT 7, VTT 1)

Evidence: VTT 7 orders the layers shape, palette, finish, style, wear,
variation, overrides, with shape "from the body's part tree or the item's
sort". Today every token's shape is its pack class's `.vox` layers. A table
may run with the sim off (VTT 1), where no part tree exists.

1. **(Recommended) The part tree is the shape; class slot layers become
   style over it** (a hood over the head part). Sim-off tables keep class
   shapes.
2. **The class wins when a token names one**; the part tree is used only for
   a token without a class.

### Q7. How orders reach a companion's mind (Directing 3, Eponym 1, Mesocosm 2; case 1130)

Evidence: ruling 60 says directives are given "as that sophont to your
friends", with opinion as their efficacy. Ruling 152 says that, outside the
world, a player directs only who they play, so everything else is asked in
the world. The sim already weighs asks and exercises of standing agreements
(`social/asks.rs`, `willing.rs`), and the contract notes that "an agreement is not
a command" (`cosmograph-overlay/src/eponym/ask.rs`). Mesocosm's nudge goes
from a participant to a critter through the bond. The founding record's
"Configure, don't command" (§1), carried into Eponym's `CLAUDE.md` as "No
real-time puppeteering of a party", is an earlier reading that orders in the
moment may contradict.

1. **(Recommended) In-world asks under a standing companion agreement.** Go,
   hold, fight and carry are works a companion agreement covers. An order
   exercises it, weighed by `weigh_routine`; outside an agreement it is a
   plain ask. An accepted order becomes a goal the companion's chooser
   follows when its own choice allows. Outcome: the founding record's rule
   is refined to "orders are asks a peer weighs", not dropped.
2. **Nudges through the bond, as in Mesocosm.** One mechanism across games,
   but it contradicts ruling 152, which would need reopening.
3. **Standing configuration only.** Gambit-style terms set in the agreement
   and no orders in the moment. It keeps "configure, don't command" whole
   but narrows case 1130's go, hold, fight and carry.

### Q8. The camera's place in isometer (Presentation 1, Stack 5; case 1131)

Evidence: `SlabCamera` is one set of numbers from which isometer derives the
tracer camera, cull window, raster matrix, body clip and picking, "so the
four cannot disagree" (`shared/isometer/src/camera.rs`). All are
orthographic. The tracer already has `TraceCamera::perspective`. Eponym
builds its own perspective matrices app-local (`scene.rs`, `residency.rs`).

1. **(Recommended) One camera, two projections.** The scene camera gains a
   projection, slab or perspective (eye, field of view, near, far). Each
   derived quantity answers for both, the presets (dimetric, first person,
   over the shoulder) are its constructors, and products only choose
   presets. A rename, if any, is a naming round.
2. **A sibling perspective type.** The scene takes either through an enum.
   The slab code stays untouched, but each derived quantity is written twice.
3. **Host-supplied matrices.** isometer takes a view and projection pair,
   as Eponym's room renderer does, with first-person and shoulder helpers.
   This is the least isometer code, but culling and residency must be
   inferred from a matrix.

### Q9. Where first person's eye sits (Bodies 15, Directing 1)

Evidence: Bodies 15 says a body knows its world by its own means, and organs
such as eyes extend it. Denizens and critters vary: an eye borne on a
segment, a sensor above the root, or no sense organ at all. Directing 3 says
a creature's senses are its own, but it suggests what it perceives.

1. **(Recommended) At the body's sense part.** Its most sense-celled part,
   facing that part's facing, falling back to the top of the root. The eye
   is where the creature knows from.
2. **At the top of the body's box, facing its heading.** Uniform and
   simple, but blind to anatomy.
3. **Per product.** Each product's preset decides.

### Q10. What direct control requires (Eponym 1, Directing 1; cases 187, 1130)

Evidence: case 187 and Directing 1 place tag-in in creative mode. Eponym's
`CLAUDE.md`, from the founding plan, says "Body switching is never a menu
operation; control changes through death or a recorded world event". Case
1130 lets the player drive a companion where its agreement allows, still
living one creature. The contract's actuation carries "never a second body".
`PLAYS` already allows several bodies per participant (cases 1062 and 1065).
A driven creature learns from its player by imitation wherever driving is
(Directing 3).

1. **(Recommended) Agreed, recorded, one body at a time.** A companion
   agreement's terms allow driving. Taking control is a recorded exercise,
   the participant then actuates the companion alone while their own creature
   acts by its own mind, and the companion may end it. Outcome: Directing 1
   and case 187 are refined, so that tag-in also lives in survival under
   agreement.
2. **Tag-in stays in creative mode.** In survival a companion is only
   ordered. This narrows case 1130.
3. **Co-drive.** The participant drives the companion while their own body
   is held still, with both played. The contract's one-body actuation
   changes.

## Decisions

None yet. Answers are recorded here as `denizen D<n>`, in answer order.

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
  denizen plan exists. The grammar and the jointed and branching foundings
  can express one (Q1's evidence).
- Isocosm's looks and tokens never read a part tree (Q4 to Q6's evidence);
  the shared scene draws `SceneVolumes::Voxels` or `DeclaredSolid`
  (`shared/isometer/src/bodies.rs`).
- isometer's scene camera is orthographic only (`SlabCamera`, and
  `isometer-render`'s `Camera`). `TraceCamera::perspective` is used today
  only by Eponym's app-local room renderer and planning camera (Q8's
  evidence). Every product builds its camera from `SlabCamera` (P1's files).
- The sim weighs asks and agreement exercises through `willing.rs`'s three
  gates, and Eponym's contract routes `EponymIntent::Ask` there; `Work`
  carries no target (Q7's evidence).
- Records that disagree with cases 1130 and 1131, for reconciliation
  once Q7 and Q10 are answered:
  - first person: the vessel briefs
    (`mesocosm/design_docs/2026-08-18_vessel_briefs_and_presentation.md`,
    "First person as a setting, Paredros only") and the founding record's
    closing note to its camera item (lines 1220 to 1226);
  - orders and switching: the founding record's "Configure, don't
    command", and Eponym's `CLAUDE.md` guardrails on puppeteering and body
    switching.

## Progress

- 2026-11-02: plan written from cases 1128 to 1131; nothing landed.
