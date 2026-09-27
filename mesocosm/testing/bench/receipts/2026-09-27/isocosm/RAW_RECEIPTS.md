# Raw receipts kept out of tree

Under ruling 255 of the wing design record, checkpoint 5's raw receipts live
outside the repository, in `Code/testing/isometry/receipts/2026-09-27/isocosm/`
on the machine that ran them. The two draw runs are one JSON line a draw and
a summary line last, as `c5-draws.rs` writes them; `c5-draws.json` here is
`python c5_summary.py c5-draws-parts.jsonl c5-draws-flows.jsonl <out>` on
them, and each `.log` holds the run's master seed as printed. The pilots are
`isocosm-probe`'s receipts on main before Part B (3af984e) and at the
checkpoint, compared with `python ../../2026-09-26/isocosm/same_but_time.py
<main> <c5>`. Check a copy against its hash before relying on it.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `c5-draws-parts.jsonl` | 288,238 | `c7611f2877a28bf046b778a72f2aa15d06d0b507dc3b935bf4f6577f46a403c1` |
| `c5-draws-parts.log` | 32 | `ef335fbc59d5d862bc6a7ef0352ecbf039b4e039a67beed67c818aa69e79a352` |
| `c5-draws-flows.jsonl` | 388,200 | `e9600f075c515d598599c9f89d4c09753cab67fcfc70f82eae04e6bf67193057` |
| `c5-draws-flows.log` | 32 | `87100a4055e9d085255c2b5350617b30a4a047bcd66c233485de80e1fd7291d5` |
| `c5-probe-pilot-food-main.json` | 108,647 | `7f211a839cad5828e70c91d8a1dc543a4fdb2f780af675199fa82c28a42d8b9c` |
| `c5-probe-pilot-food-c5.json` | 108,649 | `ad0be25253095daca9fd982d2db703a9e04329c3503c78458757fc73ef10dc9d` |
| `c5-probe-pilot-water-main.json` | 135,688 | `504d7d02fc1b87317d9310d7b3e523956c5bdba64e72f9acaeebaa5b09a6571f` |
| `c5-probe-pilot-water-c5.json` | 135,689 | `d19ea9d4ae94abd9a6d1331db0010948b7bcb3ceb502d5f62c7280275376cf0a` |
| `c5-probe-pilot-hunters-main.json` | 157,005 | `18800f87b79dfc83e2e6644e9fa98281dc873c3a77e89e7752363e83534c68df` |
| `c5-probe-pilot-hunters-c5.json` | 157,001 | `74eb6e80a3eab4b5cd9ad5d40054078c795253891cf1e089ccf4677d8c1f16e9` |
