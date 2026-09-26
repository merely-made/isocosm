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

Checkpoint 4, rulings 284 to 287, added the raw receipts below to the same
directory. The two certified checks are recomputed with `python verify.py
<copy> 1999`, whose output is `c4-*-verify.txt` here. The lineage sweep's
summary, `c4-lineages.json`, is rebuilt with `python lineages_287.py
c4-remeasure-lineages-286.json c4-remeasure-lineages.json <out>`, and the
density ladder's, `c4-density.json`, with `python c4_density.py <out>`
followed by the ten `c4-density-*.json`. `checkpoint-4-source.json` names the
commit and command behind each.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `c4-probe-predators.json` | 5,059,347 | `232dfa08aef71e27a9e764ad9b3ff30868741a187d269b302d7a59dca7230a62` |
| `c4-probe-predators-water.json` | 6,007,628 | `98ba9b34baa06c74b5a5dd681f59687ed552365dcfcd1a38bd020d730acdcfcd` |
| `c4-remeasure-lineages-286.json` | 149,442 | `5cb7578d593803b77c4b142b1d46c45512582e9650e04cda3df5525a6ca439b1` |
| `c4-remeasure-lineages.json` | 149,867 | `35dc047248a161bb863b88b52e1b5b0391523b7f3eebcdbc2b68d4d249076f06` |
| `c4-density-32.json` | 34,218 | `f99f306ad8d55be8164e6acc05249469d011dc26f4a6f41087ae225a14532d44` |
| `c4-density-64.json` | 34,318 | `db35b751df4d66bd1f4fba3388c19ca89c8c10c94e52fa9186069d25b847bd16` |
| `c4-density-128.json` | 34,365 | `7f4e350a5cb9341d20bf4a4781dcedc9255a1a753a7a884309cfeb3573b2b3f4` |
| `c4-density-256.json` | 34,541 | `0ab7cccdb9fb1c20fde4e4dc7010da2eea1fdcf9337063c9afd776c2d77bd480` |
| `c4-density-512.json` | 34,686 | `8c36feb8c437ed0d546d530480a6502b90fd046a88468c12e1d8f8674b5821fc` |
| `c4-density-plain-32.json` | 30,663 | `9d31f2824656646c9ddfaf8883b211623701a5e9152e2b10de09e596919f902d` |
| `c4-density-plain-64.json` | 30,781 | `270296619a3c3e1fd476f1c5918a689be3a68745bd789a15a2d61d90f234c39a` |
| `c4-density-plain-128.json` | 30,987 | `5241ee562fe2da670d19774e75716b72a7a036a35813d337cf7b0b806ca5035b` |
| `c4-density-plain-256.json` | 31,062 | `a27152e58c9794507db03a63f11f624391c4bb750cdcb73d2c8393c5fa92cee5` |
| `c4-density-plain-512.json` | 31,205 | `1718a71dfba25a14ac27bd77bdfd947204d459789c41d7cea51d591db9b4787c` |

The pilots that show what did not change: the probe's 16-draw pilots at seed
20260926, with food and with water, on this checkpoint's core and on
checkpoint 2's (76ab646), compared with `python pilot_compare.py <before>
<after>`; and the scale pilot at master seed 1790357350335206000 on this core
and on step 1's.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `c4-probe-pilot-16.json` | 108,272 | `7d5eca74fb901f179937c76be2a94272fe90eed3e6238aa03bf757db4b0b9c54` |
| `c4-probe-pilot-water-16.json` | 135,317 | `cf6550ac0604bf4afb026773f2e814827cd2da9fd92ca0fd842fab706aa4ae25` |
| `cp2-probe-pilot-16.json` | 107,812 | `1ef8e121eaf2b0ff7d94db181e4bfa7fcb606f7478a49bb8700dad686d7dbee9` |
| `cp2-probe-pilot-water-16.json` | 134,818 | `bfcd6de6e75e59ca930b730379aa3e6257c47426ecb648a5f62454cc2204155a` |
| `c4-scale-pilot.json` | 47,071 | `691ec13c4aac8f9bd45246a62457dd4e19034b905a37558d00edb2467ad40814` |
| `step1-scale-pilot.json` | 47,184 | `f3f5e007b9e54640962af564f6a7a940f686542118918dac371ebcb7814a423e` |
