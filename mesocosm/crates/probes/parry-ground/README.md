# Committed Ground collision receipt

This standalone R3 probe projects `mesocosm_core::places::Ground` occupancy into
one fixed conatus voxel collider (`ColliderShape::VoxelGrid`), the stack's
collider layer, with Rapier and Parry private inside it. The projection
carries the committed Ground revision and rescans only the dirty 8 by 8 by 8
Ground bricks. Until 2026-10-09 it called Parry directly; the receipts below
keep the words of their dates.

Run with:

```powershell
cargo run --release
```

The receipt compares the collider's occupancy (`voxel_filled`, `voxel_cells`),
ray (`raycast`), point (`colliders_at_point`) and ball-contact (`contacts`)
answers with Ground before and after a revisioned carve. It also refuses a
stale source revision before mutation and replays the same seed and carve into
bit-identical query answers.

The finite voxel collider covers Ground's stored `y >= 0` bricks. Ground's
implicit solid half-space below zero is a separate analytic collision shape;
materializing an infinite bedrock layer as voxels would be false economy.

## Receipt, 2026-08-21

The deterministic fixture projected 136 Ground bricks: 69,632 compared cells
and 41,763 occupied Parry voxels. A boundary carve committed revision 0 to 1,
removed 19 voxels across four dirty bricks, and caused exactly four 8-cubed
regions, 2,048 cells, to be rescanned. The other 132 region occupancy
signatures stayed unchanged.

The same committed delta moved the downward ray hit from 2.5 to 4.5 voxel
units, changed point containment from true to false, and cleared the ball
contact manifold from one contact to zero. Both a stale source revision and a
skipped target revision were refused before mutation. Re-growing the same seed
and replaying the carve reproduced the complete query receipt bit for bit.

Parry's direct ray and point traits work on `Voxels`. Contacts use the
persistent contact-manifold dispatcher; the simpler single-contact helper does
not dispatch voxel shapes in Parry 0.29 or 0.31.

## Rerun, 2026-10-09

On parry3d 0.31.1 (wing design record, rulings 692, 693 and 727), with no
code change, every figure above reproduced exactly: 136 bricks, 69,632
cells, 41,763 voxels; revision 0 to 1, 19 removed across four regions and
2,048 cells; the ray 2.5 to 4.5, containment true to false, contacts one to
zero; 132 regions retained, replay bit-identical. The probe folds into
conatus once its queries land (695, 722 to 725).

## Folded into conatus, 2026-10-09

The probe's checks moved from Parry onto conatus's `BodyWorld` queries at
mere `50fd021c` (rulings 695, 722 to 725 and 731): `refresh_queries` after
spawning and after each delta, `voxel_filled` and `voxel_cells` for the exact
comparison and the region signatures, `edit_voxels` for the delta,
`raycast`, `colliders_at_point` for containment, and `contacts` for the ball.
Every figure reproduced: 136 bricks, 69,632 cells, 41,763 voxels; revision 0
to 1, 19 removed across four regions and 2,048 cells; the ray 2.5 to 4.5,
containment true to false, contacts one to zero, the deepest at -0.3; 132
regions retained, replay bit-identical. The probe's name predates the fold;
a rename is a naming round.
