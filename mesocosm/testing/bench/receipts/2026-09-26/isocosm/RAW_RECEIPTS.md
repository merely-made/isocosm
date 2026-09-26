# Raw receipts kept out of tree

Under ruling 255 of the wing design record, the three large raw receipts of
checkpoint 2 live outside the repository, in
`Code/testing/isometry/receipts/2026-09-26/isocosm/` on the machine that
ran them. Their summaries (`*-verify.txt`), the run source
(`checkpoint-2-source.json`) and the recomputation script (`verify.py`) stay
here. Check a copy against its hash before relying on it, then recompute with
`python verify.py <copy> 1999`; on review the recomputation matched each
`*-verify.txt` exactly.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `probe.json` | 3,809,321 | `036088b1ca3adbb71e9659b5849ec077caf1973b2367dcbe8b36f345c8c20490` |
| `probe-water.json` | 4,592,230 | `80297cc0ed66720a31e3b8a31fcedbca33fcebbab3e08b4d44f7ae810281b562` |
| `probe-approximate.json` | 4,786,398 | `2a5e9b759893d96b65283d510105d3cd186c21520f29e6e6cba17788fa3b8fa5` |
