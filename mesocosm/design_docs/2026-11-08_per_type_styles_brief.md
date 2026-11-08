# Per-type styles: rendering each sort of thing by configuration

**Status:** 2026-11-08: design brief. Mark asked for it with case 1183 ("Bodies
now, brief the rest"). The same day, case 1182 wired declared style tags for
bodies (§2). This brief is not a plan and changes no code. Its questions are
in §8.

**For:** Mark, on 2026-11-08, answering the colour-path fork (1178): "it
would be cool if we could differentially render each sort of entity just as a
matter of technical configuration... could make for interesting visual styles
for mods that change the presentation of the game in other ways too (e.g. 2D,
2.5D, 3D, etc. different perspectives, generated from the data which defines
the assets/entities)".

This brief covers how far that idea reaches: styles for terrain, bodies and
marks; projections derived from entity data; what mods could change; and what
it costs. Paths are relative to the repository root.

## 1. The idea, narrowed

A *style* is how one sort of thing is drawn. It is never what the thing is.
The rule the wing already holds for looks also holds for styles: the sim never
sees them. Terms (TG7), the VTT's looks and now body styles all sit outside
native rules, saves and digests.

There are three layers:
- **Tag.** A pack names a style, such as `film` or `baseline`, and says what
  it changes.
- **Rule.** A product says which of its things carry which tag. It matches on
  facts the product already has: kingdom, lineage or role in Mesocosm, a
  token's class in the VTT, an actor's kind in Eponym.
- **Resolution.** The tag is resolved when the scene is projected. The scene
  keeps only the resolved style, never the rule.

## 2. What exists now

| Seam | Where | What it varies today |
| --- | --- | --- |
| Style tags (1182) | `shared/isometer/crates/isometer-core/src/style.rs` (`Style`, `StyleTable`); `mesocosm/crates/mesocosm-phenotype/src/styles.rs` (rules) | A body's `BodyDisplay`, matched on kingdom, lineage or role; first match wins. Unknown tags, roles and looks are refused at admission. |
| `BodyDisplay` (1178) | `isometer-render` instance lane; `SceneHost::body_display` | Per body: exact sRGB, or Neutral tonemap then 2.2 power. Part of each sprite's appearance revision. |
| Presentation 2's rungs (1161, 1171) | `HybridPolicy`, `SceneHost::hybrid_held` | Per body: live voxels or a cached sprite, by projected size and held detail. Marks and aggregates are rungs not yet built. |
| `HybridSettings` (1161, 1177) | `shared/isometer/src/hybrid/settings.rs` | One quality preset per host: bands, refresh periods, image size, byte budget, light step. |
| VTT looks (doctrine VTT 7) | `crates/isocosm-views/src/appearance.rs`, a pack's `appearance.toml` | A class name binds voxel layers, slots and a palette. Tokens override per instance, and packs overlay in order. |
| Tilesets as stylesheets | `crates/isocosm-views/src/lib.rs`, `board.rs` | The VTT board is a DOM. Tiles, props and tokens carry CSS classes, so a tileset is a stylesheet. |
| Terrain grade | `isometer-lens` `Grade::clay` and `Grade::retro`; Mesocosm `TerrainStyle` | One look for the whole map: smooth, or palette-quantised with ordered dither; fog. |
| Projection | `SlabCamera` (orthographic slab or perspective); `CameraMode` (dimetric, isometric, oblique, side, terrarium); `View` (overview, over-the-shoulder, first person) | One camera for the whole scene. |
| Generated 2D | `isometer-mesh` `bake_facing` and `bake_strip`; the sprite atlas | Voxel data baked to sprite sheets, per facing. |

What the table shows:
- Bodies are the only things with a per-thing style.
- Terrain and the camera are styled per scene, not per thing.
- The VTT already has the data-driven class table the idea asks for, but it
  binds geometry, not drawing.

## 3. Styles across terrain, bodies and marks

**Bodies.** Done for one knob (§2). The next knobs that fit the same `Style`
record:
- the rung preset, meaning the `HybridSettings` bands and refresh;
- the sprite image size;
- a palette remap;
- an outline or a flat-shading switch in the live pass.

Each knob is a field with a baseline default. That is how `display` works
today, so a pack names only what it changes.

**Terrain.** Terrain is not drawn per entity. It is a brick map traced in one
pass, so a per-type terrain style keys on material: soil, rock, water, or a
site's own matter. The tracer already reads a material palette and a grade. A
style table keyed by material could choose:
- the palette entry;
- the grade, by distance band;
- a dither strength.

A per-material grade means a branch inside the tracer, not a second pass.

**Marks.** Marks (Presentation 2's third and fourth rungs, and the `wing-glyphs`
icons) are not built for bodies yet. When they are, a mark style chooses the
glyph set, size and outline. The same tag could select the body's mark, so a
lineage that reads as `film` up close also reads as its mark far away.

## 4. Projections derived from entity data

The quote's 2D, 2.5D and 3D map onto what the stack already draws:
- **3D** is the live rung: voxel geometry from the body document, under any
  `SlabCamera`.
- **2.5D** is the sprite rung: the same geometry baked per facing and drawn on
  current roots with depth. Isometric pixel-art boards are this.
- **2D** is a flat sprite or a DOM element with no depth join. The VTT board
  is already a 2D projection of tokens, from CSS classes.

So "a projection per type" means three things:
- a rung forced by style rather than chosen by projected size;
- a camera mode chosen per scene;
- for true 2D, a board-like DOM projection fed by the same entity data.

The first is small: a style field, say `rung = "sprite"`, that pins a body's
rung. The selector already takes a held-detail bound, so a pinned sprite is a
held bound the pack supplies. The second exists, per scene. Per type, it would
mean several cameras in one frame. The third is the VTT's board, which would
need a Mesocosm view crate on the same shape.

Deriving the asset from entity data already happens. Body documents carry
anatomy, and `bake_facing` turns voxels into sheets. A style that chooses the
bake (facings, scale, palette) changes the 2D asset without changing the
entity.

## 5. What mods could change

A mod is a pack overlay. Pack styles let it change:
- which things carry which tag (rules);
- what each tag means (looks);
- a scene's grade and camera defaults, if those become pack data (they are
  CLI flags and code defaults today).

It cannot change rules, saves or digests. That holds already, because styles
are admitted beside terms and never enter Genesis.

Mesocosm loads one pack today (`app.rs`, `pack_root`). The VTT's
`Library::overlay` already orders several packs' looks. Styles would need the
same: later packs win per tag, and rules concatenate with the later pack
first.

## 6. One table shape across products

Case 1182's consolidation check:
- **The VTT's `Library`** binds geometry: layers, slots, palettes, items and
  swatches. It is the wrong owner for drawing styles, and folding styles into
  it would bind VTT item machinery to Mesocosm.
- **The shared piece** is the tag-to-look table (`StyleTable`, `Style`,
  `BodyDisplay`). It now lives in `isometer-core`, the pure crate all three
  products already depend on. Each product keeps its own rules, because what
  a rule matches on is product vocabulary.
- **The VTT can adopt it** by adding `style = "<tag>"` to an `appearance.toml`
  class entry and a `[styles]` table beside it. That adds no second copy.

## 7. Costs

- **Draw cost.** Per-body styles ride an instance lane. The cost is one float
  per instance and a branch in the fragment shader, and nothing per frame
  beyond that.
- **Batching.** The live pass batches by volume and palette, not style, so
  styles do not split batches. A style that changes geometry, such as outlines
  or flat shading, would add a pass or a pipeline.
- **Images.** A style is part of an image's appearance revision, so a style
  change rebakes that body. A pack whose rules flip often, for example by role,
  rebakes on every flip. Role changes are rare, but lineage-keyed styles at
  speciation would rebake a whole lineage at once, inside the existing refresh
  budget.
- **Terrain styles.** A per-material grade adds tracer work in every terrain
  pixel. That needs a measurement against the existing tracer budget before
  anything is promised.
- **Several cameras.** A per-type camera multiplies the scene's draw. That is
  the second-camera machinery Presentation 3 plans for minimaps; using it for
  styles costs a full scene pass each.
- **Authoring.** Every new knob is a field that every pack's validator must
  check, and a look that tests must pin. Closed sets, such as roles and the
  display words, keep that small.

## 8. Questions for Mark

*Answered 2026-11-08: question 1 as case 1185, 2 as 1186, 3 as 1187, 5 as
1188, each as recommended. Question 4 is open.*

1. **What the next body style knob is.** Today a style changes only
   `display`. Evidence: the rung, image size and palette are already per-body
   inputs or one step from it (§3).
   - Recommended: a pinned rung (`rung = "live" | "sprite"`), which is the
     2.5D-per-type case and reuses the held-detail bound.
   - A palette remap per tag.
   - Outline or flat shading, which needs a new pass.
2. **Whether terrain gets per-material styles.** Evidence: the tracer already
   reads a material palette and a grade per scene (§3). Per-material grade
   costs work in every terrain pixel, which is unmeasured.
   - Recommended: measure first, as a tracer receipt on the M4.7 scene, before
     any terrain style table.
   - Build a per-material palette override only, with no grade change.
   - Keep terrain styled per scene.
3. **How a 2D projection is reached.** Evidence: the VTT board is a 2D DOM
   projection from CSS classes; Mesocosm has no view crate shaped like it
   (§4).
   - Recommended: pinned sprite rungs under an orthographic camera first, for
     a 2.5D look per type with no new renderer.
   - Write a Mesocosm board projection on the VTT's shape, for true 2D.
   - Defer 2D.
4. **When the VTT adopts the shared table.** Evidence: `StyleTable` lives in
   `isometer-core`. The VTT's native board already submits its tokens to the
   shared pass (`crates/isocosm-views/src/scene/board/native/source.rs`), so
   `body_display` reaches them. The VTT's `appearance.toml` binds classes to
   geometry (§6).
   - Recommended: when the VTT first wants a style, as a `[styles]` table
     and a `style` field on class entries. The shape is ready; nothing is
     wired speculatively.
   - Adopt it now, with a baseline tag for every class.
   - Keep VTT looks and styles separate.
5. **Whether scene defaults become pack data.** Evidence: grade, camera mode
   and the default body style are CLI flags and code defaults; a mod can only
   reach them through code today (§5).
   - Recommended: yes, as an optional `[presentation]` table, beside `terms`,
     that a pack overlay can set.
   - Keep them as player settings, not pack data.
