# Raw receipts kept out of tree

## 2026-09-27 additions

Later independent gates retain their own source and hash manifests:
[traversal bands](bands.md), [current omissions and local settings](settings.md),
and the test-only [atlas replacement experiment](replacement.md). Their
directories are under `Code/testing/isometry/receipts/2026-09-27/`; the
2026-09-26 receipts below are preserved as recorded.

Ruling 373 adds the [one-submission replacement measurement](batched-replacement.md)
in `lane-e-atlas-batched/`, preserving the first experiment and draft run.

Under ruling 255 of the wing design record, the raw logs of the scene
board's paging receipts live outside the repository, in
`Code/testing/isometry/receipts/2026-09-26/scene-board-paging/4663de8/`
on the machine that ran them; paths below are relative to that folder. The
summary (`summary.md`) and the scripts that run the receipts and rebuild the
summary (`receipts.py`, `headed.py`) stay here. Check a copy against its
hash, then `python receipts.py --summarise-only --out <dir>` rebuilds the
summary from it. An earlier round's files stay where they were, hashed by
the manifest committed with that round.

Source: commit `4663de87422a471d793245d5cf8f674843cf89f6`, test `paging_receipts_at_256` in
`crates/isometry-views/src/scene/paging_tests.rs`, run with
`--ignored --nocapture --test-threads=1`, and `--release` for the release log.
Each log opens with its commit, command and compiler; each headed session's
`source.txt` names its commit, binary hash and switches.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `debug.log` | 42,333 | `44b39611f39d5ccc027dc6841ea7c03445ebb8c65ba67ea3062ba1936572094b` |
| `release.log` | 42,827 | `9a567787ba47eaf68ec5c3689c938803c393f7ede2a7bd811936ce63f3d4c751` |
| `host-1-4663de8/headed-demo/stderr.log` | 116,128 | `a78f0d2065bba7aa939b71e31eac6415249eba0e4018f2472ad2c68bad1f5b50` |
| `host-1-4663de8/headed-demo/isometry_capture.png` | 369,892 | `a161727ae2bc2bc4812aa8ae4ba47441872816aee432c1c504658cce583c4a47` |
| `host-1-4663de8/headed-demo/source.txt` | 252 | `b8274e35b63e53c3107255c6bc03413c998cb823403f6973387aefa5f8b538d2` |
| `host-1-4663de8/headed-256/stderr.log` | 92,792 | `0b67acc0b863ee6030aecd24cba76b94d05ef01c2a58eda1bee2941ca2d5ea65` |
| `host-1-4663de8/headed-256/isometry_capture.png` | 725,962 | `cebcb9a61f06d5cafb8231527119b2542a234695476534981c17f2e7be43b0c8` |
| `host-1-4663de8/headed-256/source.txt` | 271 | `5e1d8dccbb7cf591aca7e59c2e33be67be45030f4958b3680588ac11abda415b` |
| `host-2-4663de8/headed-demo/stderr.log` | 118,621 | `0eb13e73cd1545c7698ecc257b59f1c714981fdc7842cd9fd2df796abe25df21` |
| `host-2-4663de8/headed-demo/isometry_capture.png` | 369,892 | `a161727ae2bc2bc4812aa8ae4ba47441872816aee432c1c504658cce583c4a47` |
| `host-2-4663de8/headed-demo/source.txt` | 252 | `b8274e35b63e53c3107255c6bc03413c998cb823403f6973387aefa5f8b538d2` |
| `host-2-4663de8/headed-256/stderr.log` | 94,790 | `a835da0652539d97694a8aec6c0807ce4524582c569fa8c98e1ac0bb835b5422` |
| `host-2-4663de8/headed-256/isometry_capture.png` | 725,962 | `3917e1419ee1b02357354b65824e9fc485f0b9fcd5e8baf7a18677e5dc47e922` |
| `host-2-4663de8/headed-256/source.txt` | 271 | `5e1d8dccbb7cf591aca7e59c2e33be67be45030f4958b3680588ac11abda415b` |
| `host-3-4663de8/headed-demo/stderr.log` | 121,137 | `ab740c277f0cc2f091e75b51e8be10537d87fea66b7ddd347208e88287da461f` |
| `host-3-4663de8/headed-demo/isometry_capture.png` | 369,892 | `a161727ae2bc2bc4812aa8ae4ba47441872816aee432c1c504658cce583c4a47` |
| `host-3-4663de8/headed-demo/source.txt` | 252 | `b8274e35b63e53c3107255c6bc03413c998cb823403f6973387aefa5f8b538d2` |
| `host-3-4663de8/headed-256/stderr.log` | 94,829 | `f3ce25a5e1f1876c9cf291cd9372b6ca73eded1cece6d8dbff23dfba7bbe0b74` |
| `host-3-4663de8/headed-256/isometry_capture.png` | 725,962 | `cebcb9a61f06d5cafb8231527119b2542a234695476534981c17f2e7be43b0c8` |
| `host-3-4663de8/headed-256/source.txt` | 271 | `5e1d8dccbb7cf591aca7e59c2e33be67be45030f4958b3680588ac11abda415b` |
| `headroom-check/headroom-0-run-1/stderr.log` | 75,401 | `3970cf6eb3f294294981094dd2bc0a9e29102c8876843d7d7f0509a3b69b1278` |
| `headroom-check/headroom-0-run-1/isometry_capture.png` | 726,012 | `8da8c71b2841db38754baffe4753d899bee465abd10485fd80d8e74217660166` |
| `headroom-check/headroom-0-run-1/source.txt` | 297 | `affffad84cf034246c49925c19ef31343cadd7faa4d73a0a96fc0a95b99152bc` |
| `headroom-check/headroom-0-run-2/stderr.log` | 45,225 | `0cc00cfe3e16cf088a8223dc4b8a28864c7e78e0e0b49e26fada524426b52e7c` |
| `headroom-check/headroom-0-run-2/isometry_capture.png` | 726,012 | `8da8c71b2841db38754baffe4753d899bee465abd10485fd80d8e74217660166` |
| `headroom-check/headroom-0-run-2/source.txt` | 297 | `affffad84cf034246c49925c19ef31343cadd7faa4d73a0a96fc0a95b99152bc` |

## 2026-09-27 final integration batch

Ruling 374 selects upfront allocation of the chosen feasible budget. The
combined-source suites, fresh GPU gates, 150 cost records, and six successful
headed captures are recorded in [integration.md](integration.md), with exact
source qualifications and the raw manifest hash. [final-cost.md](final-cost.md)
contains this batch's regenerated release tables. Earlier raw directories and
annotations above remain unchanged.
