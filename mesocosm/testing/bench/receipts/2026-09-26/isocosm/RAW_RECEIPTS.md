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

The scheduler index of ruling 258 added raw receipts to the same directory.
The lineage sweep's summary is `scheduler-lineages.json`, rebuilt from the
six re-measures with `python scheduler_lineages.py <out> <before-1> <after-1>
<before-2> <after-2> <before-3> <after-3>`; `scale-lineages.json` is their
source, the lineage points of `../../2026-09-25/isocosm/scale.json` alone, as
`trim_lineages.py` writes them. `remeasure-ecology-258.json` is the 30
ecology points run again on the new core. `scheduler-source.json` names the
commit and command behind each.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `scale-lineages.json` | 147,616 | `574629403c236c69697de2f0328bacea226c836aa9fd05c869dff6fb7a5edb66` |
| `remeasure-lineages-before-1.json` | 150,385 | `c86614bb0a9d3f0e28fa13bbeccba2d7d8057437b59af958ecfe56e16c632f4c` |
| `remeasure-lineages-after-1.json` | 150,498 | `da08a43c0704469899dd8447d450ad0adaac918ceb33c6f8150db29bee40e7ef` |
| `remeasure-lineages-before-2.json` | 150,362 | `2386a850edbfadea930d44dbba89e2131206270a7aeddce4bec76ef741c8835d` |
| `remeasure-lineages-after-2.json` | 150,378 | `33701dba96892ac89568bd34a05b02f6ae81689f747ccb69f63109646dbfa4c8` |
| `remeasure-lineages-before-3.json` | 150,350 | `842efe40bf459f03bfacfc95e553ab928329bcb643ee1d3e59c5d2ab894e2887` |
| `remeasure-lineages-after-3.json` | 150,339 | `0c189eb92f0ceb6511c3770faa754c6c40b7edb7e5c9fcdd31c7e6687c21c40c` |
| `remeasure-ecology-258.json` | 431,322 | `03f8ebd61b27374cff36fa7c36231ec2d3d255aea72f17a63c784290caaa6aa8` |
