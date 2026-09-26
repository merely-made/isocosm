# Raw receipts kept out of tree

Under ruling 255 of the wing design record, the raw logs of the scene
board's paging receipts live outside the repository, in
`Code/testing/isometry/receipts/2026-09-26/scene-board-paging/` on the
machine that ran them. The summary (`summary.md`) and the script that runs
the receipts and rebuilds the summary (`receipts.py`) stay here. Check a
copy against its hash, then `python receipts.py --summarise-only --out
<dir>` rebuilds the summary from it.

Source: commit `31370bfa9d16923c7ef16621411e63818a3a449d`, test `paging_receipts_at_256` in
`crates/isometry-views/src/scene/paging_tests.rs`, run with
`--ignored --nocapture --test-threads=1`, and `--release` for the release log.
Each log opens with its commit, command and compiler; each headed session's
`source.txt` names its commit, binary hash and switches.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `debug.log` | 38,702 | `20679e3134e7c681a53a6542378b4d39fa9aa9322853637db707b8525ab5773e` |
| `release.log` | 56,653 | `b89c305d6f152783bc8a051046a4aa25f89346c358f54941c8f80ad5575fdeea` |
| `headed-demo/stderr.log` | 81,039 | `d1c0697d5c054a3948e2ec361e62a5b2a486270ce5754c73f01a9ee16ab68c8d` |
| `headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `headed-demo/source.txt` | 286 | `25d0dfb348059ad807d0e7fde0b6577a587f07a3931aa243a480721a96327794` |
| `headed-256/stderr.log` | 66,925 | `88e9e2e2b2051aa123b12ede28f216a457124d03035d8777e5ad62df335a2867` |
| `headed-256/isometry_capture.png` | 719,305 | `2e59478eb02aae54ae81abc9ca6916db44e94bc61cb99449f992d7197e8ac48a` |
| `headed-256/source.txt` | 305 | `acf529c79aa21fda71e7c3faafbbb50598c093b8e80249069ecba0a338b39be2` |
