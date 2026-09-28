# SP1 raw receipts, 2026-09-28

The spatial spine's SP1 (place-graph engine plan §A.5 and §A.8, rulings 393
to 399). Source commit: `6f25a89` (full hash in `sp1-source-commit.txt`).
The files below are retained under
`Code/testing/isometry/receipts/2026-09-28/isocosm/` (ruling 255).

`sp1-map-draws.json` is `isocosm-bench --map-draws 256` in release, its
master seed `1790634037465385300` taken from the clock, not chosen. It drew
256 worlds: 83 planes, 81 rings and 92 tori, 2 to 16 sites wide and high,
sites 256 to 2,048 base units a side. On every one, 79,156 border sides in
all read the same profile bytes from both sides, 83,656 corners agreed from
every slot around them with each border ending at its corners' heights, and
founding the world twice gave one digest.

`sp1-tests.log` is the full suite at the source commit: 121 passed, 0
failed, the 111 of the baseline plus ten in `tests/spine.rs`. Two of those
ten run permanent controls that must fail and assert that they do: a profile
keyed by the ordered pair of sides fails the symmetry check, and a corner
height keyed per site fails corner agreement. A third is a positive check:
corner walks find a grid's exact corner count on every shape, (W+1)(H+1) on
a plane, W(H+1) on a ring and W·H on a torus, with every slot in exactly one
corner. It exists because the per-site control alone could not prove the
walk: a walk returning single slots would still fail that control, through
the border-end check.

`sp1-control-direction.log` is a deliberate break: the far side made to read
its border forwards (`let forward = true;` in `terrain/profile.rs`). The
corner check fails, "site 1 side 3 misses its corners", exit 101. The source
was then restored with `git checkout` and `git diff` was clean.
`sp1-clippy.log` has no warning; `sp1-fmt.log` is empty, the check passing.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `sp1-clippy.log` | 152 | `aeafd12334b9e4ccde0d7e579fb46c98424fe86e2ffcecb980df5d139e2297cf` |
| `sp1-control-direction.log` | 1315 | `eef9a9c32c02ea002af8ddbb3003fa4029d1567b03fb520275ade40172e81d3f` |
| `sp1-fmt.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `sp1-map-draws.json` | 113743 | `d25daf4239d38007e68e9f0ee8009eaf27b44868ae7531d26dc406048b501d67` |
| `sp1-map-draws.stderr.log` | 149 | `ab05ef961138dbdca36c6ea412ac5052fce9415456a0c0fc90d433606c58d356` |
| `sp1-source-commit.txt` | 41 | `b1b382ee9fb05084276ada358adfb06eb41d77d4fd46655ff631a10b9f63ca28` |
| `sp1-tests.log` | 11685 | `41f3d0aae4a45063d35d9e3773923fabdfeaa242a3bcd725948f59ff82b05e3f` |
