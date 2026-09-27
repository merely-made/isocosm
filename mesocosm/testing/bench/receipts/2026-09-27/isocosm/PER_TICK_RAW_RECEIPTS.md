# Ruling 371 raw receipts, 2026-09-27

Source commit: `41d30b66055268b7f9e524042c02df2ad3169b97`. The files below are retained under
`Code/testing/isometry/receipts/2026-09-27/isocosm/`. They preserve complete
fresh stdout/stderr; older checkpoint-5 raw files were not changed.

`c5_per_tick_summary.py` rebuilds `c5-per-tick.json` from the new flow JSONL
and the historical `c5-draws-flows.jsonl`. The four differential stdout files
match the historical hashes after CRLF-to-LF normalization. Actual file
hashes below preserve their on-disk bytes.

The development flow log precedes the final both-mode rollback check. The
111-test suite predates the one-line Rust 1.98 Clippy syntax update in
`probe/exact.rs`; the final Clippy gate, 10 probe tests and differential
runs follow it. The initial Clippy log records that lint failure. The two
control logs deliberately fail tests against temporary faulty source; exact
source restoration is recorded in `per-tick-mutation-controls.json`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `c5-per-tick-differential-build.log` | 659 | `63506d1539543a076a2a80a0c1838b68c06d478ee3d67894db71d48eeab06d68` |
| `c5-per-tick-draws-build.log` | 647 | `9f80ad4e42d5828402d21cfce293e37e8f2a50cec8b0a093c7a3ae5055ae7847` |
| `c5-per-tick-flows.jsonl` | 431516 | `94c06799a12b5daeb94abb9aaef82f241bf98b2e42125a365307534a9b88095a` |
| `c5-per-tick-flows.stderr.log` | 32 | `87100a4055e9d085255c2b5350617b30a4a047bcd66c233485de80e1fd7291d5` |
| `per-tick-clippy-final.log` | 479 | `238d57800300905c2ec60e0f84f2191cab13368ed4fb092591faa9cc8a775aeb` |
| `per-tick-clippy.log` | 1984 | `eb035cd703008f09708ae8898e9f7e509abb547e630a04f1eddd48d9e0b3f04d` |
| `per-tick-control-missing-move.log` | 1510 | `0952ce2ff1753e2ee79c6085ac6df4ca658e7cc98f66567c6f6a208993e2751d` |
| `per-tick-control-retained-moves.log` | 1387 | `1041c8d1b684bf2b9794997518ee1466c3a83226054e4b580e50606c4667812f` |
| `per-tick-differential-fuzz-11.stderr.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `per-tick-differential-fuzz-11.stdout.log` | 50404929 | `012e917571e203f2ce479b8a45914278e9a8b17f94f56d73efa3118fe7b71c5b` |
| `per-tick-differential-fuzz-7.stderr.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `per-tick-differential-fuzz-7.stdout.log` | 5697880 | `9ae1af613968f20f3a921625884db380ea22d4ebf50da0849badc76cfbd96af9` |
| `per-tick-differential-probe-20260927.stderr.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `per-tick-differential-probe-20260927.stdout.log` | 4045 | `5cc9fdf39686b7e51e42e381c9a238ee8e117ded010f9876a2e5d3a35ed82224` |
| `per-tick-differential-runs.json` | 3493 | `81950cb7a32024809008d913237afa94c720e8fa487e2e6ee8413c350b2f7d37` |
| `per-tick-differential-tight-13.stderr.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `per-tick-differential-tight-13.stdout.log` | 13807289 | `8a8e1ab6cb1dc650c9ef909cc262d6a5bd9c7cad6dca7952ca8f52a0adedbb54` |
| `per-tick-draw-run.json` | 578 | `291d722388f1e82e0d8a60ace5fb1669b2f3bebbab7f152f61f2f040625acd31` |
| `per-tick-driver-builds.json` | 1802 | `4cc35f4f8072124bac4e20d8f2c2f7578432e1707eed69aeb5d7a71b283a4a8d` |
| `per-tick-flow-tests-development.log` | 2165 | `b6689ff126dfcd1a51b46b1a41073eb508acbf37f2956ddee2a472237babc0b2` |
| `per-tick-fmt.log` | 197 | `7d85ed00a8b85c5025113190584ad4e163063f92030f4c7dfc0eb02ab492c7b7` |
| `per-tick-gates.json` | 12687 | `93ba009892c8dfd842b404c66db6b606a43f5615f05b37399f2c9551ec370156` |
| `per-tick-mutation-controls.json` | 1060 | `477e551c2a309381914601cfe63f69bfb416b00140a9a432758d2ecf95d6c670` |
| `per-tick-probe-final.log` | 1299 | `2dca58ffa59dd432278467fb762c34286e7be2380674a92bccf4254d5a8add3f` |
| `per-tick-release.log` | 1603 | `bd198e49e671537ad27922293e850f8ff530a847bccd867416ada8f83078422f` |
| `per-tick-summary.log` | 1753 | `872b040b8c2bfa77ae8914609413ad4985e0a7d9e1fed0353d6a356d4261eb0d` |
| `per-tick-test.log` | 11299 | `8a10a4bd7ff33acd0448b3013b95e7d6138ddec8efdbf09fdda998fc3d848fdd` |
