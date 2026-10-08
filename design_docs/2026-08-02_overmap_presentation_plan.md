# Overmap Presentation Plan: Hulls, Backdrop, and a Map That Reads as Terrain

**Date:** 2026-08-02
**Status, 2026-09-26:** Rewritten to the record 2026-09-26 (ruling 280). *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status:** Active, with **two prerequisites reopened by the 2026-08-08
audit** ahead of the "already landed" list:
(1) **a neutral region-paint seam**: sprigging's `GraphCanvas` privately
owns paint order and geometry, so "extend the existing leaf" is not
implementable as written; a product-free region/composite layer is now
justified by the Mesocosm minimap as second consumer;
(2) **hulls derive from final displayed node positions**, including local
placement overrides, never from authored coordinates alone. Required
receipts: uniform-position, unplaced-node, override, parallel-route, and
a headed screenshot.
**Product direction recorded (wing session, 2026-08-08): source-time as a
feature** — the overmap viewed from historical standpoints: what was
believed then, what the table knows now, what was retconned, which map
version a character possessed. This is the wing's *claim* carrier at
campaign scale; it rides this plan's machinery once the prerequisites
land.
**Companion:** mere `design_docs/mere_docs/implementation_strategy/2026-07-21_projection_proofs_plan.md`
(the arrangement register), mesocosm's minimap (`mesocosm-views`, the first
Hulls consumer and the working example to follow).

**Rewritten to the record, 2026-09-26 (wing design record ruling 280).**
W1 (2026-09-18) evaluated this plan as mixed, sim place graph and game
overlay, and found it contradicted on one point and confirmed on the
other. Contradicted: §3.1 sites the hulls on the overmap nodes' authored
`at` positions, where the record's §3.7 and its ruling 14 derive places
from the volume and hold the world map as adjacency, hexes and node
positions being a projection over it (sim plan §2.2; rulings 72, 73, 18).
Confirmed: source-time as a feature is the record's own model, the reach
field of §3.4 and the sim plan's §4 (rulings 5, 84, 117). This pass rewrites
the sources (§3.1, §3.5, §4) and leaves the presentation as planned; its
done-conditions are authoritative again. The evaluation stays in
[mesocosm/design_docs/2026-09-18_wing_plan_evaluations.md](../mesocosm/design_docs/2026-09-18_wing_plan_evaluations.md)
§1. The overmap is a far view (ruling 212): it reads the crowd, so what
lands here is presentation over the sim's readings, and what the campaign
takes from the sim lands under the
[VTT overlay plan](2026-09-25_vtt_overlay_plan.md)'s V2.

---

## 1. What this is

The overmap swatch is contract-native (P4 landed: `overmap_score` →
`scenomise::solve` → scene → `GraphCanvasSwatch`) but its presentation is
ball-and-stick: dots, straight edges, labels, no sense that a site *sits in*
a region of the world. Mark's brief: shape hulls around nodes to give a sense
of node-associated territory, put them against a backdrop, and be cleverer
about data representations generally (mosaic subgraphs for inventories are
the standing example).

This plan upgrades the overmap's presentation without touching campaign
truth, discovery (E6), or the score/scene contract's product-freedom.

## 2. Already landed (do not rebuild)

- **`sceno::Arrangement::Hulls`** (mere, 2026-08-02): bounded nearest-site
  partition. Every coordinate-placed item is a site; each cell is the bounds
  clipped by perpendicular bisectors; the solver emits one `sceno::Region`
  per site with a `Footprint::Polygon` contour. Cells tile the bounds.
- **`sceno::Region`** was already the scene face of the hulls lane; the
  solver now fills it.
- **`numen::FieldExtent::Polygon`** (mere, 2026-08-02): containment + signed
  boundary distance, persisted through graph-kernel. Available when a region
  needs scripted/inferred meaning; not required for presentation.
- **`mesocosm-views`** (mesocosm, 2026-08-02): the first Hulls consumer.
  `minimap.rs` is the adapter shape (disclose → solve → resolve meaning
  vessel-side); `leaf.rs` is a sprigging `Leaf` that paints region polygons,
  site dots, and a position marker from a solved scene. Read both before
  writing isometry's version.

## 3. The work

### 3.1 Hulls behind the overmap

Add a Hulls realization to the overmap swatch: territory cells around the
discovered sites, painted under the existing nodes and edges.

- **Adapter**: a second score (or an extended `overmap_score`) with
  `Arrangement::Hulls`. Sites are the discovered overmap nodes' authored
  `at` positions; bounds fit the discovered extent with a margin. Only
  discovered nodes are sites: the unfound map is not drawn, and a cell for
  an unfound site would leak its existence.
  **Rewritten 2026-09-26:** the sites are the world map's own nodes, held
  as adjacency (sim plan §2.2, rulings 72 and 73), and their positions come
  from the projection the overmap already solves, never from an authored
  coordinate (ruling 14; the 2026-08-08 audit's second prerequisite said
  the same from the presentation side). "Discovered" is what the party's
  characters know by reach, possibly wrong (rulings 84, 117; VTT overlay
  plan §3 point 4 and V2), the DM seeing the truth; `party_known` is the
  seam that reading replaces. The leak rule stands unchanged: a cell is
  drawn only for a site the party knows.
- **Meaning is per-vessel** (ruled 2026-08-02): isometry tints by
  campaign facts it already owns. First cut: faction control where a
  faction claim exists, biome/terrain kind otherwise, neutral where neither.
  The contract carries geometry; `WorldPlace`/faction resolution stays in
  `isometry-campaign`.
- **Paint**: follow mesocosm's `MinimapLeaf` (translucent fill, full-strength
  boundary, cells under nodes). Either extend the existing overmap leaf or
  paint regions in the same leaf before nodes. Do not add a second leaf key
  unless layering forces it.
- **Isometry caveat**: overmap cells are *derived* territory, not simulation
  truth (unlike mesocosm, where the partition rule is the world's own).
  Present them softer: lower fill alpha, or `Region.confidence` mapped to
  opacity. Do not let derived territory read as authored borders.

### 3.2 Backdrop

DOM layer under the leaf (ruled 2026-08-02: dynamically generated, not
static; DOM unless effects force painting into the leaf).

- The overmap swatch sits in a DOM element; give it a background layer
  generated from campaign data. The natural isometry backdrop: a baked
  low-res isometric render of each site's tactical map (the voxel bake
  pipeline already produces sprites), composited as region-anchored images,
  or a single generated terrain wash from biome facts.
- Start with the cheapest honest version: a generated (not shipped-asset)
  terrain wash tinted from the same per-region facts as 3.1, as CSS. Baked
  map thumbnails are a follow-up once region anchoring is proven.
- Tilesets-are-stylesheets applies: the backdrop binds through CSS class
  vocabulary so campaigns can reskin it.

### 3.3 Region interactivity

Cells are hit targets: click a cell to select its site (same action as
clicking the node), hover to show the region's facts (faction, biome,
travel weight of routes touching it). Follow the graph-canvas pattern:
paint stays in the leaf, hit targets are native DOM positioned from the
same solved scene, so a11y and keyboard come free. The polygon hit test
can use the region contour; a bounding-box approximation is acceptable for
the first cut if the precise test fights the DOM.

### 3.4 What NOT to do

- Do not compute layout in isometry-views. The solver owns placement; the
  adapter disclosed facts. Hand-rolled geometry is the debt P4 just deleted.
- Do not put faction or biome vocabulary into sceno/scenomise. The audit
  rule stands: neither portable crate mentions any product.
- Do not draw undiscovered territory, even unlabeled. Discovery is a
  campaign rule, not a presentation choice.
- Do not block on Mosaic/Atlas. They are reserved arrangements with their
  own future consumers (inventories; geographic). This plan is Hulls only.

### 3.5 Source-time, on the reach field (added 2026-09-26)

The product direction recorded 2026-08-08, the overmap viewed from
historical standpoints, is what the record's reach field already holds
(record §3.4; sim plan §4; rulings 5, 84, 86, 117). Each place an event
reaches keeps its arrival: the tick, the source it came by, its strength
and **the version that arrived**. So the four readings the direction named
are four questions to one field, none of them a second store:

| Reading | The question |
| --- | --- |
| what was believed then | which version had reached the party's places at that tick, by the arrival entries |
| what the table knows now | the same at the present tick, decayed under the world's rule, the legend floor included |
| what was retconned | the difference between a held version and the record's, a belief being dated by its arrival (117) |
| which map version a character possessed | a bearer (a map, a charter) holding a version at a tick, the record's "more than the place" |

Nothing here is built by this plan. The field is the sim's (sim plan S1,
S3); the overmap consumes it as a far view through the VTT's side of the
overlay contract at V2. What this plan owns is drawing those readings with
the same hull and backdrop machinery, a standpoint being one more input to
the adapter.

## 4. Done conditions

- The overmap swatch shows territory cells around discovered sites, tinted
  by vessel-owned meaning, under the existing nodes and edges.
- A generated backdrop renders under the cells as a DOM layer, restylable
  via CSS class vocabulary.
- Cells are clickable and answer hover with region facts.
- `cargo test --workspace --all-features` green; the sceno audit tests
  (type names stay `sceno::`) untouched and passing.
- A headed scenario screenshot in `Code/testing/isometry/` showing the
  upgraded overmap, per the screenshots harness convention.
- **Added 2026-09-26:** the sites and their discovery come from the sim's
  place graph and reach field through the overlay contract (VTT overlay plan
  V2), not from authored `at` positions or `party_known`; the campaign the
  screenshot shows is a draw under a seed nobody chose (ruling 15); and one
  capture shows the overmap from a past standpoint differing from the
  present one on at least one site's version (§3.5).

## 5. Open questions (ask Mark, do not decide unilaterally)

1. Faction tint vs biome tint when both exist: layered, blended, or
   faction-wins?
2. Should undiscovered edges of a discovered cell be clipped hard (fog
   boundary) or faded?
3. Baked map thumbnails as region backdrops: worth it now, or after the
   wash proves the layering?

All three still open on 2026-09-26; none is decided by the record. Question
2 gains a fact: an undiscovered edge is a site no version has reached, so
"fog" is the reach field's zero, and the choice is only how to paint it.
