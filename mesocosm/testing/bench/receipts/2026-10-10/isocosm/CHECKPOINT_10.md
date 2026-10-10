# Checkpoint 10, 2026-10-10: independent slice

**Ruling 810, 2026-10-10:** healing restores each lost cell's recorded
previous function or free-pool status, preserving authored allocation and
inherited variation. Q8 resumes healing and tombstone revival; their
implementation and receipts remain open. The earlier envelopes below
retain their measured scope and do not certify this new work.

Q8 is in progress. This receipt covers native wounds, severing, fragment
matter, agentless hazards and rot. Healing and tombstone revival wait for
the restored-function allocation ruling. It does not certify checkpoint 10.

The measured source is base `870d16b3` plus the explicit file hashes in
`q8-harm-restored-source.json`. Raw evidence is at
`Code/testing/isometry/receipts/2026-10-10/isocosm/`. The original envelope
`q8-harm-source.json` is preserved; the restored envelope links it and the
controls to their exact source and repaired test inputs.

| Raw evidence | SHA-256 |
| --- | --- |
| `q8-harm-source.json` | `9af521e80fa5de0ae9c79cef0d6d200803dffe715a4602b6cba2421913be7f91` |
| `q8-harm-restored-source.json` | `b489ccf141d6b4d6cca04624b1f10d73e4006bd7b3792a0a86a3f145854179e0` |

The restored source passed `cargo check --workspace --all-targets --offline
-j1` in `shared/isocosm` and `cargo test --offline -j1 --test harm`: ten
tests passed. Wounds and severing reconcile each holder and account against
their recorded flow legs. Tests cover both native execution modes, empty
parent tombstones, fragment matter, root death, lowered
account bounds, save/load, population-limit refusal, cell-weighted part
draws, rate-zero and certain hazards, rot, and tombstone/draw-slot admission.

Two planted faults compiled and were caught: duplicating fragment matter
instead of emptying parent ledgers, and making rate-zero hazards wound.
`q8-harm-controls.json` preserves commands, original/mutant source hashes
and full failing outputs. Original bytes were restored, followed by the
passing check and tests. `c10_faults.py` defines further controls; only
indices 0 and 5 have run in this receipt.

The generated harm domain has one agentless world actor per site, one
local target and one harmful operation per process. Its current-state
target read is equivalent to its pass-start read in that bounded domain.
Subsequent source-only changes explicitly reject competing actors,
cross-site selectors, risk/notes and multiple harmful operations, and
dispatch commitments as well as effects. Those changes and their five
new tests, an identity-target rejection test and expanded fragment birth assertions have not yet run. Generic crowd harm passes remain outside this
receipt.

Remaining gates: verify those crowd guards, implement healing and revive
tombstones, run all touched workspace gates, catch the remaining planted
faults, and independently recompute a fresh crowd certification. Historical
checkpoint 9 receipts are a format reference, not fresh proof.

## Bounded guard checkpoint

The next batch used integrated base `f6976e85` and the thirty file hashes
in `q8-scoped-source.json`. Its native all-target check, sixteen harm tests
and the repaired route-cut test passed. The route-cut fixture now severs
the whole subtree, retains empty tombstones and transfers every removed
holding into its site. Fault 9 compiled and was caught: removing the crowd
multiple-actor guard made its rejection test fail. Every envelope file
matched after restoration. `q8-scoped-controls.json` records the command,
original/mutant source hashes and full output.

The final bounded source defers ordinary HEAL catalogue/life assignment;
FRAGMENT assignment remains active under 713. This changes ordinary
founding's genesis serialization and digests. The opt-in harm probe retains its dormant HEAL marker.
This is a bounded checkpoint, not a new ruling or completed healing.
Only `bodied.rs` changed from the measured guard batch. Final source in
`q8-fragment-source.json` passed another native all-target check, all
sixteen harm tests and founding trait/bounds coverage. Its guard and test
bytes remain identical to fault 9's inputs.

| Raw evidence | SHA-256 |
| --- | --- |
| `q8-scoped-source.json` | `b56c1a00ae05fc6f68ccdd6c4cec67ff015212326ca994b6a070e5cfc0f939bb` |
| `q8-scoped-controls.json` | `049eac1e10b31b25b6f0032db3aa26337bada09dba39ad8f3d76f26b63cf1753` |
| `q8-fragment-source.json` | `706b35e05f673f48c312da9864a7fad7b21729483771bf5211aa67977c0bbfd2` |

The new batch verifies the crowd scope guards and committed effects;
generic crowd harm remains unsupported. The previous receipt still
describes its own earlier source. Consumer gates, healing/revival, the
remaining planted controls and fresh independently recomputed crowd
statistics remain open. Checkpoint 10 is not certified.

## Integrated consumer compile checks

Root merged `9b0ae46d` onto `12a26c13` and recomputed all 27 code/test/script
hashes: each matches the tested checkpoint bytes. All-target offline checks
passed for root/all-features (38.30 s), Mesocosm (22.25 s),
Eponym/all-features (29.06 s), wing-integration (24.38 s) and parry-ground
(49.57 s). `q8-root-integration.json` records the gate input tree, exact
commands, environment paths, source/log hashes and unchanged native lock.
Its SHA-256 is `ceb4640437ed444e40b23f540d03fd62b85e5bfc0214d32c8c1453a9bff4aefa`.
These compile checks close this checkpoint's downstream compile window;
healing/revival, the remaining faults and full statistical certification
remain open.
