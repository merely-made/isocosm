# SP2 raw receipts, 2026-09-28

The spatial spine's SP2, the lift (place-graph engine plan §A.5 and §A.9,
rulings 400 to 403). Source commit: `38ea90f` (full hash in
`sp2-source-commit.txt`). The files below are retained under
`Code/testing/isometry/receipts/2026-09-28/isocosm/` (ruling 255), beside
SP1's.

`sp2-lift-draws.json` is `isocosm-bench --lift-draws 64` in release, its
master seed `1790635299382812200` taken from the clock, not chosen. It drew 64
worlds from the map draws' space: 25 rings, 21 planes and 18 tori, 2 to 16
sites wide and high, sites 256 to 2,000 base units a side. In them:

- 18,386 border sides gave exactly the same surface as their neighbours at
  every base-grain point along the border;
- the first and last site of every world, 128 sites, had their exact surface
  summed column by column over all 195,970,560 base-grain columns, and every
  sum was the site's elevation times its column count, so the closed-form
  correction holds against brute force;
- all 4,905 sites kept every lattice point's detail within their relief;
- 1,891 sampled chunks, at the base grain and three levels up, lifted to the
  same bytes twice.

`sp2-tests.log` is the full suite at the source commit: 133 passed, 0 failed,
the 121 of SP1 plus twelve in `tests/lift.rs`. A second process repeats a
lift's digest there. Three of the twelve run permanent controls that must
fail and assert that they do: detail that does not fade at the edges parts
the borders, a lattice without its correction misses the mean, and detail
drawn past its window breaks the relief bound. The water test asserts that
some drawn columns lie under water, so its water branch is exercised and not
merely passed.

`sp2-control-direction.log` is a deliberate break of the border check
itself: the far side read in the same direction as the near one (`let
across = t;` in `terrain/check.rs`). It fails, "site 0 side 1 parts from its
neighbour 0 along", exit 101. The source was then restored with `git
checkout` and `git diff` was clean. `sp2-clippy.log` has no warning;
`sp2-fmt.log` is empty, the check passing.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `sp2-clippy.log` | 152 | `ffd34d939062b13fef2556118b21639c97190b8b5087cd1d06c0c3ba5b5ceef6` |
| `sp2-control-direction.log` | 874 | `3623424ee37938d66668eb781b3d0e8e84935d41c21b50f57c5d4d87ade8df7d` |
| `sp2-fmt.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `sp2-lift-draws.json` | 31016 | `912a763c38104a768c0dfe68fb3bb6c3aa8732c44a5fa17935e18551ed8b46f8` |
| `sp2-lift-draws.stderr.log` | 162 | `a54f3c586180ccaebb4828c569f7da304287edfea7c28fec9121077dd6e40c24` |
| `sp2-source-commit.txt` | 41 | `13a5721a66238214ef5571f12bc669264401d7ed84636db94c26ae1c27348d41` |
| `sp2-tests.log` | 12489 | `8e43741699daaadaa941e624d4f072f04f53700ac6b3de2b654f86fcc84b7f78` |
