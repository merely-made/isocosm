# Raw receipts kept out of tree

Under ruling 255 of the wing design record, the raw logs of the scene
board's paging receipts live outside the repository, in
`Code/testing/isometry/receipts/2026-09-26/scene-board-paging/3312d08/`
on the machine that ran them; paths below are relative to that folder. The
summary (`summary.md`) and the scripts that run the receipts and rebuild the
summary (`receipts.py`, `headed.py`) stay here. Check a copy against its
hash, then `python receipts.py --summarise-only --out <dir>` rebuilds the
summary from it. An earlier round's files stay where they were, hashed by
the manifest committed with that round.

`set-aside-noisy-window/` beside these holds a first headed round taken
while other lanes loaded the machine, its demo controls reading up to
16.08 ms. It is kept, and not summarised; the rounds below replaced it.

Source: commit `3312d0883adebb51a4d95dbb601d2cae639c99d4`, test `paging_receipts_at_256` in
`crates/isometry-views/src/scene/paging_tests.rs`, run with
`--ignored --nocapture --test-threads=1`, and `--release` for the release log.
Each log opens with its commit, command and compiler; each headed session's
`source.txt` names its commit, binary hash and switches.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `debug.log` | 42,786 | `d0931465fa9429d69661aa6d5dd16bd57a297333bcb6a5cdbf0b007496fc1ab3` |
| `release.log` | 42,958 | `cc630c925d37429ac73ef8b8960e343ede1851bb1bd8bd84f7723517f2ffe97e` |
| `host-1-31370bf/headed-demo/stderr.log` | 102,780 | `96983c28dff7a9730f51f2f3d98008c5e137cf16d3ff1e1eb60804260a025038` |
| `host-1-31370bf/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-1-31370bf/headed-demo/source.txt` | 252 | `c29ec9699c7f76e6849d19a66b42db02dea55ac4ccdcdecb5f8cdd2df38d718f` |
| `host-1-31370bf/headed-256/stderr.log` | 75,742 | `c2801cb609369e02c954a3f70b1f2bf2b7a3c83066ff9ccffe61da461654758a` |
| `host-1-31370bf/headed-256/isometry_capture.png` | 719,305 | `2e59478eb02aae54ae81abc9ca6916db44e94bc61cb99449f992d7197e8ac48a` |
| `host-1-31370bf/headed-256/source.txt` | 271 | `00484ef09c13e4d858b6022af6411a94bd2584e25763b430d0fb106efeb6fb39` |
| `host-2-c6fb846/headed-demo/stderr.log` | 107,085 | `f014097eaaaa9c66d5b28ace8bf22ade492dab3f80c3ca53aafadb0e1746065a` |
| `host-2-c6fb846/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-2-c6fb846/headed-demo/source.txt` | 252 | `e8e4c6e3ab99cd90f2115adee62169a7477bb27483d795dbd24334ffe6f1d685` |
| `host-2-c6fb846/headed-256/stderr.log` | 87,902 | `89fe7ed98e410be9cea0d3fbbf9d47f1d6347685251e737366e7e758c8c18ea2` |
| `host-2-c6fb846/headed-256/isometry_capture.png` | 719,306 | `3aac8291eab2400f884b550847c8aabd3ff4c4ba64b715720a9772e04374a97e` |
| `host-2-c6fb846/headed-256/source.txt` | 271 | `9557d0c7362f84b97dfb0f0ca8d16572d1ab4fc71a924333d86c2e908f34745c` |
| `host-3-3312d08/headed-demo/stderr.log` | 108,570 | `2f57422cb119ba49ebb335ecf7f419f93f041b0ffce6f99577d0ff6bfdf42110` |
| `host-3-3312d08/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-3-3312d08/headed-demo/source.txt` | 252 | `2f82c7d3de84b80b8dda611a587a1dac72e7f7cc7c4931915302169b6f752cfd` |
| `host-3-3312d08/headed-256/stderr.log` | 90,403 | `f0141037ff437b03e03c45a377fb27bd2782c240ab69396ce01a82dde7d984e9` |
| `host-3-3312d08/headed-256/isometry_capture.png` | 719,256 | `080556e2b4a3d23a5d5f4262cd2f2c27d954bdd3338b996ddaf60960806ad6e0` |
| `host-3-3312d08/headed-256/source.txt` | 271 | `7556c84d7f06e104c53ae4adf6e63a0e6e54d734bc07eaff9e3185de8353b809` |
| `host-4-31370bf/headed-demo/stderr.log` | 118,829 | `728726fad4b9f7506a114a0f8312d44afadeb4838aa5ef814cf22ceb84fadcfc` |
| `host-4-31370bf/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-4-31370bf/headed-demo/source.txt` | 252 | `c29ec9699c7f76e6849d19a66b42db02dea55ac4ccdcdecb5f8cdd2df38d718f` |
| `host-4-31370bf/headed-256/stderr.log` | 75,212 | `e1051bce6dd807ec126e62008960a2fa13b43c0fb0c14eea9743ca690de942d1` |
| `host-4-31370bf/headed-256/isometry_capture.png` | 719,305 | `2e59478eb02aae54ae81abc9ca6916db44e94bc61cb99449f992d7197e8ac48a` |
| `host-4-31370bf/headed-256/source.txt` | 271 | `00484ef09c13e4d858b6022af6411a94bd2584e25763b430d0fb106efeb6fb39` |
| `host-5-c6fb846/headed-demo/stderr.log` | 119,104 | `121a9400d5725e02d6792dd687d2fe0c8ab67f43fd0fce5afc678ee78d2441ca` |
| `host-5-c6fb846/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-5-c6fb846/headed-demo/source.txt` | 252 | `e8e4c6e3ab99cd90f2115adee62169a7477bb27483d795dbd24334ffe6f1d685` |
| `host-5-c6fb846/headed-256/stderr.log` | 96,412 | `11f26e513741de60475284d9ceb88033dfb724f96b959b77ef767f76c7c50058` |
| `host-5-c6fb846/headed-256/isometry_capture.png` | 719,306 | `3aac8291eab2400f884b550847c8aabd3ff4c4ba64b715720a9772e04374a97e` |
| `host-5-c6fb846/headed-256/source.txt` | 271 | `9557d0c7362f84b97dfb0f0ca8d16572d1ab4fc71a924333d86c2e908f34745c` |
| `host-6-3312d08/headed-demo/stderr.log` | 114,078 | `8781d0bc5ea108b8717763c5544f0f958024ce0c55c02b39653da60552e7b959` |
| `host-6-3312d08/headed-demo/isometry_capture.png` | 371,378 | `57dc102202c22a3a50182acea3ffd7be974ad30b3e9c8949e89d61059ae4b8fe` |
| `host-6-3312d08/headed-demo/source.txt` | 252 | `2f82c7d3de84b80b8dda611a587a1dac72e7f7cc7c4931915302169b6f752cfd` |
| `host-6-3312d08/headed-256/stderr.log` | 89,423 | `66be7e483faead361ce9f9e73edfb82be63b3fdb430c9d131d9fd5a6d5665dc4` |
| `host-6-3312d08/headed-256/isometry_capture.png` | 719,255 | `3791168b2bd776853f2a5554a2212458f285458cfadd1af4c574fea917eed3cf` |
| `host-6-3312d08/headed-256/source.txt` | 271 | `7556c84d7f06e104c53ae4adf6e63a0e6e54d734bc07eaff9e3185de8353b809` |
| `headroom-check/final-headroom-0-run-1/stderr.log` | 96,465 | `cca4689f53c275267e013187a9e80f1a305845283247cd260a662c20f37581fc` |
| `headroom-check/final-headroom-0-run-1/isometry_capture.png` | 719,305 | `2e59478eb02aae54ae81abc9ca6916db44e94bc61cb99449f992d7197e8ac48a` |
| `headroom-check/final-headroom-0-run-1/source.txt` | 297 | `58661de4ad0a8098403f36719eb60b50753b4507983d2605f0acb2797fbdf69f` |
| `headroom-check/final-headroom-0-run-2/stderr.log` | 89,987 | `e63141537c37da9651cf5a6ac539e4c8baf1d3248c7dec71c0f0850d8e8336c0` |
| `headroom-check/final-headroom-0-run-2/isometry_capture.png` | 719,305 | `2e59478eb02aae54ae81abc9ca6916db44e94bc61cb99449f992d7197e8ac48a` |
| `headroom-check/final-headroom-0-run-2/source.txt` | 297 | `58661de4ad0a8098403f36719eb60b50753b4507983d2605f0acb2797fbdf69f` |
| `headroom-check/final-headroom-0-run-3/stderr.log` | 87,441 | `580c3bad7b77e506caeb6aa390e1625e46a06353d1b2fd4ad38cbf4043cdca84` |
| `headroom-check/final-headroom-0-run-3/isometry_capture.png` | 719,306 | `3aac8291eab2400f884b550847c8aabd3ff4c4ba64b715720a9772e04374a97e` |
| `headroom-check/final-headroom-0-run-3/source.txt` | 297 | `58661de4ad0a8098403f36719eb60b50753b4507983d2605f0acb2797fbdf69f` |
| `headroom-check/final-headroom-0-run-4/stderr.log` | 88,448 | `2aa0ecdf1a75cbb4004c6e0227b0076dcfdd366bc84c502a2d9299a3a0850bbc` |
| `headroom-check/final-headroom-0-run-4/isometry_capture.png` | 719,306 | `3aac8291eab2400f884b550847c8aabd3ff4c4ba64b715720a9772e04374a97e` |
| `headroom-check/final-headroom-0-run-4/source.txt` | 297 | `58661de4ad0a8098403f36719eb60b50753b4507983d2605f0acb2797fbdf69f` |
