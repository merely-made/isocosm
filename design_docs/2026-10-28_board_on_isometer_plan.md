# The board on isometer

**Status:** 2026-10-28: cut from `2026-09-15_board_on_isometer_plan.md`,
`2026-09-25_vtt_overlay_plan.md` (the board and appearance items),
`2026-08-08_extracted_receipts.md` (headed acceptance, tile geometry), which are
archived in `archive_docs/2026-10-28/`. The authored scene board is built behind
the `ISOMETRY_SCENE_BOARD` flag beside the DOM board, with per-tile diffing and
paged residency. Open are the DOM board's fate, the sim-on lifted-site path,
what the scene board does not yet draw, the appearance theory, the smallest
supported display and headed acceptance.

**For:** drawing the VTT board through the shared isometer scene while keeping
the locked 2:1 lens, the tileset-as-stylesheet contract, and every tile and
token selectable; settling the DOM board's fate; the scene lowering, selection
and presentation gates of the sim-on battlemap; the board's visual and
reachability residue. The decision numbers 1 to 4 below are kept.

Not here: the sim-on path itself, its assertions, replay and convergence
(`vtt-overlay V2`); camera freedom beyond the shipped lens (a lens other than
the 2D lens needs its own plan, per CLAUDE.md); the detail ladder, far marks and
the overmap as a second camera (`orthographic-voxel-presentation`); isomere
(`sim-host`); the session model (`session-network`); campaign packs
(`content-packs`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Decisions

Copied verbatim from the old plan's section 5.

## 5. Decisions for Mark

1. **Tokens live or billboarded.** The plan takes ruling 6 at its word:
   live bodies from recipes (I4), sprites kept for a later far tier. The
   alternative is a camera-facing image batch beside the glyph batch, cheaper
   and closer to today's look, and it is what a hybrid would add later.
2. **Pixel grid ownership.** Internal resolution and integer scaling move
   from the DOM's `board_scale` to the producer's render scale (I3). The
   GBA crispness pillar then holds inside the scene rather than in CSS.
3. **What overlays wait for isomere.** Reach, path and selection tints can
   land through materials now; richer overlays (range templates, facing
   arcs, labels) are isomere's and would otherwise be built twice.
4. **The DOM board's fate** after parity: archive, or keep as a downlevel
   path.

## Constraints

- **Eight-way movement, one diagonal cost.** Binding, preserved from the
  archived tile geometry seam plan (2026-07-17): movement is eight-way and one
  diagonal costs one (Chebyshev), so `reachable`, melee range and shove
  direction agree. Archived text: "Mark chose diagonal movement, so `reachable`
  now steps all eight neighbours and one diagonal costs one, exactly as
  Chebyshev says." Pointcrawl is not part of the tile seam. The test
  `budget_caps_the_reach` asserts the Chebyshev shape. Source:
  `archive_docs/2026-08-08/2026-07-17_tile_geometry_seam_plan.md:98-100`.
- **The lens stays the shipped 2D lens**, the 2:1 dimetric (doctrine
  Presentation 1). Other lenses need their own plan.
- **Tileset-as-stylesheet is the modding contract.** The class vocabulary
  `tile-<kind>`, `token-<sprite>`, `prop-<kind>`, `cond-*`, `beat-*` is what a
  tileset targets (PROJECT_DESCRIPTION pillar 3). The scene board must keep
  the stylesheet deciding looks.
- **Selectable means** what the clip receipt
  `board_tile_clips_its_hit_area_to_the_visible_diamond` defines.
- **Isometer additions are tested inside isometer** against its existing
  receipts, which stay byte-identical where the new inputs are unused.
- The `isometry-core` crate stays pure and keeps iso math and `depth_key` for
  any DOM overlay that still wants a paint order.

## Phases

### BD1. Restate the done-conditions against the current geometry
The old plan's done-conditions were parity with the DOM board and a constant.
On 2026-09-18 the wing review (W1) said "rewrite": the lanes landed and the
code is good, but the wing record's section 3.6 rewrites the geometry under
them, and "until it lands this plan's done-conditions are not authoritative".
Whether that rewrite lane happened is not shown. This phase restates what the
board must satisfy (probe-pixel parity of selectable tiles, the clip receipt,
no board elements emitted, overlays visible) against the geometry now in code,
and records which of the old B-lane conditions still hold.
- Done when: not stated; owed
- Needs: the wing design record section 3.6 (archived 2026-10-22);
  `orthographic-voxel-presentation`
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:14-19`.

### BD2. The DOM board's fate (decision 4)
Decision 4 is held open by Mark's ruling of 2026-09-16 ("decide later") until
the family's brick cap was resolved: "The switch is re-decided when isometer's
owner rules on paging or a larger cap". Paging landed (rulings 288 to 292,
368, 369, 372, 374), so the stated trigger has fired. The scene board's steady
frame profile is about 2.6 times faster than the DOM board's on the same
machine. The one fact that still favours keeping the DOM board is that it is
the only arm that works with no usable adapter. The ruling is a question for
Mark (see Questions).
- Done when: (B5) "Done when the scene board's frame profile is recorded beside the M4 and atlas numbers on the same machine, the headed session shows zero board elements emitted, and the DOM board is archived with rationale or kept behind the flag by Mark's ruling." The profile and the zero-element record exist; the ruling does not.
- Needs: Mark's ruling
- Rulings: 282, 288 to 292, 368, 369, 372, 374
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:178,192-193,382-391`.

### BD3. The sim-on lifted-site board
With the sim on, a battlemap is a lifted site (799) drawn on the scene board:
projected from the generated volume in the game's grid with the DM's map as an
edit over it (243, 89). This plan owns the scene lowering, selection and
presentation gates of that path; `vtt-overlay V2` owns the path (lift, held
edits, assertion replay).
- Done when: not stated; owed
- Needs: `vtt-overlay V2` (phase 3, battlemap and replay); BD2 if the scene
  board is to be the default
- Rulings: 799, 243, 89
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:5-12`;
  `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:27-29`.

### BD4. What the scene board does not draw
The scene board does not draw flat-sprite props (they stay DOM and are
counted), the `alt` checkerboard shade, or the DOM's `:hover` tile highlight (a
board with no per-tile element cannot have one). An overlay tint on the scene
board is about 9% darker than the same hex on the DOM board, because the tracer
shades a top face (`#ffd766` arrives as 231, 196, 92). A one-pixel column in
both arms tracks the window. These matter only if the scene board becomes the
default (BD2) and depend on the appearance theory (BD5).
- Done when: not stated; owed
- Needs: BD2, BD5
- Rulings: 6, 14
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:419-429`.

### BD5. The VTT's two theories of appearance
Owed 2026-10-22 from the design record's section 8: the stylesheet theory
(tileset-as-stylesheet, the modding contract) and the voxel-sourced theory.
The colour table now generates the CSS. The old overlay plan called it "a
question for this plan to rule"; it is held here, and it is a question for Mark.
It bears on BD4's tints and on `content-packs` (appearance preview, a real
tileset).
- Done when: not stated; owed
- Needs: Mark's ruling; BD4
- Rulings: PROJECT_DESCRIPTION pillar 3
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:332-334`
  (the "Owed" row of its section 6).

### BD6. Richer overlays and the far, many-token tier
Decisions 1 and 3: tokens are live bodies from recipes (ruling 6). The baked
sprites are kept for "a later far and many-token tier" (a camera-facing image
batch beside the glyph batch). Reach, path and selection are material tints
now; range templates, facing arcs and labels wait for isomere so they are not
built twice. Neither the hybrid tier nor the isomere overlays are built.
- Done when: not stated; owed
- Needs: isomere (`sim-host`); `orthographic-voxel-presentation` for the far
  tier
- Rulings: 6, 14
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:80,119-123,182-191`.

### BD7. Improvements recorded for isometer's owner
Recorded in the old plan, not owned by this repository's board code, and kept
as a list so the owner has it: `Cutaway` cuts bodies only (a slab plane in the
tracer would make it one concept); `HostTerrain` has no incremental path;
`Ground` has no material write (a one-tile tint regrows about 33,000 voxels);
nothing asks "will this fit" before a growth or reports a brick budget or bytes
per brick; `Terrain::surface` encodes "nothing here" as a negative y;
`MAX_MATERIAL` (63) and `MAX_TERRAIN_MATERIALS` (64) are one bound counted
twice with no compile-time link; `SceneFrame::capsules` and `SceneHost` are
Mesocosm seams and `always_visible` is named for Mesocosm's critter; the
heightfield march and chain critter sit unconsumed in isometer-lens; the bake's
four facings against continuous yaw (the `Clip` vocabulary is reserved but
empty). Several are overtaken by the paging work (see Findings).
- Done when: not stated; owed
- Needs: isometer's owner (`shared/isometer`)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:195-228`.

### BD8. Remove the dead focus branch in `BoardGround::sync`
`BoardGround::sync` has a branch for "the focus moved but the revision did not",
which the old plan says cannot fire because `Overlays` carries the focus. It
costs nothing; remove it after confirming it cannot fire.
- Done when: not stated; owed
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:336-339`.

### BD9. Reachability at the smallest supported display
Every control must be reachable at the smallest display the VTT supports. No
smallest display is named yet (a question for Mark). The design-height figure
itself is settled: 820 logical pixels (`DESIGN_SIZE`), and
`the_side_panel_fits_the_design_height` asserts the last row at or above 800.
- Done when: "every control reachable at the smallest display the VTT supports" (no display named)
- Needs: naming the smallest supported display
- Rulings: S60 (mere)
- From: `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:398-409`.

### BD10. Headed acceptance of the board and table
Headed receipts the archived plans listed and never landed, merged into one
acceptance item: headed screenshot coverage from the bootstrap plan;
interaction and windowing receipts from the viewport and windowing plan; the
exploration plan's one missing headed receipt; the gameplay roadmap's C7
(factions as participants) receipts. The bootstrap plan's real-network session
proof (two machines on different networks, mid-session join, matching log
hashes) is not run here: it waits on the network gate in `session-network H7b`
and the overlay's headed two-peer receipt in `vtt-overlay V2`. The gameplay
roadmap's distribution residue goes to `content-packs`.
- Done when: not stated; owed
- Needs: `session-network H7b` and `vtt-overlay V2` for the network part;
  verify each against its archived source before picking it up
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-08_extracted_receipts.md:17-19,22-23,32-33,39-40`;
  `archive_docs/2026-08-08/2026-07-05_isometry_bootstrap_plan.md` (I4);
  `archive_docs/2026-08-08/2026-07-07_viewport_windowing_and_chrome_plan.md`;
  `archive_docs/2026-08-08/2026-07-18_exploration_mode_plan.md`;
  `archive_docs/2026-08-08/2026-07-14_gameplay_roadmap_plan.md` (C7).

## Deferred and parked
- **Genet text over the scene (labels, overlays)**: not built; see BD6 and the
  question on "Genet text acceptance". Waits on isomere. From:
  `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:10,469`.

## Questions for Mark
- **The DOM board's fate (decision 4)**: archive the DOM board with rationale,
  or keep it as a downlevel path behind the flag. The old plan's options are
  exactly "archive, or keep as a downlevel path". Since the paging trigger has
  fired, the ruling is now due. From:
  `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:178,382-391`.
- **Stylesheet or voxel-sourced appearance (BD5)**: which of the two theories
  governs how the board gets its looks, now that the colour table generates the
  CSS. The old plan stated the two theories, not a recommendation. From:
  `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:332-334`.
- **What "Genet text acceptance" means**: the old plan names it open in its
  status line and last progress entry but never says what it is. The wing
  record (case 438, `2026-09-18_wing_design_plan.md:4370-4395`) put three
  options for a far mark's glyph display: (A) pixel icons from the display,
  (B) Genet text over the scene, (C) strokes per glyph; Mark chose A. Does the
  phrase mean acceptance of Genet-drawn text over the scene (labels, overlays),
  and which ruling does it wait on? From:
  `archive_docs/2026-10-28/2026-09-15_board_on_isometer_plan.md:10,469`.
- **The smallest supported display (BD9)**: which display size is the floor
  for "every control reachable". From:
  `archive_docs/2026-10-28/2026-09-25_vtt_overlay_plan.md:398-409`.

## Findings
- 2026-10-28: the scene board is behind `ISOMETRY_SCENE_BOARD`
  (`crates/isometry-views/src/board.rs:311`, `state.rs:118`,
  `crates/isometry-genet/src/scene_routing.rs`); the DOM board has not been
  archived or made the fallback by ruling. The sim-on lifted-site path does not
  exist.
- 2026-10-28: the board uses `SlabCamera::dimetric_2_1` (I5) in
  `crates/isometry-views/src/scene/world.rs`, and B3 retired `tile_at_pane` and
  `token_drag_candidate` as reachable paths (`scene/parity_tests.rs`,
  `state/interaction/gestures.rs`). Decisions 1 to 3 are as built: tokens are
  live bodies, the producer owns the render scale, tints go through materials.
- 2026-10-28: the old item "page the CPU side too" (ruling 290) is done.
  `scene/ground.rs` reads the map as per-tile columns, diffs per tile and holds
  only the bricks the frame shows. The old claim that a board past
  `modulus::MAX_BRICKS` (2,047) is refused is stale: the test
  `the_brick_cap_bounds_residency_not_the_board` builds boards at 72, 80, 96 and
  the generator's edge (`MAX_GENERATED_MAP_EDGE`, 256) and asserts each holds
  only the frame's bricks. The "nothing warns the author" worry no longer
  applies. This removes the cap as a reason to keep the DOM board.
- 2026-10-28: of BD7's list, `Ground::material(at)` now exists
  (`shared/isometer/crates/isometer-core/src/ground.rs:386`) and the full
  regrow per overlay change is replaced by the per-tile diff; the rest was not
  re-checked.
- 2026-10-28: `MAX_TERRAIN_MATERIALS` is 64 (`isometer-lens/src/lib.rs:126`),
  as the old plan says.
- 2026-10-28: the eight-way, one-diagonal-cost test `budget_caps_the_reach` is
  in `crates/isometry-core/src/path.rs:159`.
- 2026-10-28: `DESIGN_SIZE` is (1100, 820) in `crates/isometry-genet/src/main.rs:168`.
- 2026-10-28: the old plan's timing and single-sample figures (M4 layout median,
  regrow times) are machine-specific and are not carried as thresholds.

## Progress
- 2026-10-28: cut from the board-on-isometer plan, the VTT overlay plan's board
  and appearance items, and the extracted-receipts ledger.
