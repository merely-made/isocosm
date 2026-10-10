# Q2 native Character metadata, 2026-10-10

Ruling 809 retains optional table cell and participant-owner metadata on native
Character without control or body movement. This receipt covers the native
checkpoint only. Consumer routing, campaign folding, peer replication, the
compiled discarded-metadata control and product compile/runtime gates remain
pending. The pure overlay's earlier 46/46 receipt is separate.

## Inputs and ownership

Worktree: `C:\Users\mark_\Code\worktrees\isometry-contracts`, branch
`lane-contracts`, starting HEAD
`702ab6cad622dd3f31d3656f4bcc5eb149fcccc2`. Prior native checkpoint:
`9dd71d78dd84fadc3b4fb0ecdb1e06248a914737`. Common base:
`c6bc153a19efa6e6f8dd10b0154814958dfffc9f`.

The tests ran the combined native snapshot of 19 source paths below, before
refreshing Q3/Q8 native work from main. This checkpoint changes 11 native paths
beyond the prior native checkpoint. Unverified consumer WIP was preserved.
The bounded Cargo slot ran from 22:25:10 to 22:34:59 UTC and was then released.
All commands used the offline cache, one Cargo job, and these stable outputs:

```powershell
$env:CARGO_TARGET_DIR='C:\t\cargo-targets\isometry\contracts'
$env:CARGO_BUILD_BUILD_DIR='C:\t\cargo-targets\isometry\contracts\build'
```

Native `Cargo.lock` was unchanged, SHA-256
`9417E959E1E6B254E4F240EE10E7C7E639B642C71BCA3B1459386684AA029321`.
No downloads, temporary target or separate Cargo home were used.

## Gates

| Command | Result | Compile/check | Runtime |
| --- | --- | ---: | ---: |
| `cargo test --manifest-path shared/isocosm/Cargo.toml --lib asserted::tests:: --offline -j1` | 9/9 passed | 3m37s | 0.03s |
| `cargo test --manifest-path shared/isocosm/Cargo.toml --lib simulation::preimage::tests:: --offline -j1`, first attempt | 3/4 passed; fixture precondition failed | not retained | not retained |
| Same preimage command, corrected fixture | 4/4 passed | 24.31s | 0.01s |
| `cargo test --manifest-path shared/isocosm/Cargo.toml --test witness old_saves_ --offline -j1` | 0/2 passed; known Q4 failures below | 3m27s | 0.00s |
| `cargo check --manifest-path shared/isocosm/Cargo.toml --workspace --all-targets --offline -j1` | passed, no diagnostics | 22.56s | n/a |

The first preimage attempt selected the first generated entity, which was a
world entity without a body. The test
`all_unchanged_entity_fields_and_assertion_variants_keep_their_native_encoding`
failed its positive `entity.body.is_some()` precondition at line 281. Fixture
selection was corrected to find a generated entity with a body; body/part
positive assertions stayed in place and production source stayed unchanged.
The failed test-file SHA-256 was
`F8A56FA73D3AFDC8C51824D9CB48575B13DE8894FA9A79C11DA46FF7BD0C823C`.
The corrected and passing SHA-256 is in the source table below.

The unchanged historical-fixture failures were:

- `old_saves_load_and_save_again_as_v3`, `tests/witness.rs:75:68`: unwrap of
  `Err("save version or genesis digest mismatch")`.
- `old_saves_are_checked_at_every_checkpoint`, `tests/witness.rs:109:17`,
  Individuals: actual `"save version or genesis digest mismatch"`, expected
  `"checkpoint history mismatch"`.

These are the same genesis-gate failures in the dated Q4 baseline and Q8's
8/10 historical-witness run. They were preserved without changing readers or
bypassing validation. They do not invalidate the separate constructed former
Character v3 archive control, which passed replay in both execution modes.

## What the passing controls establish

The assertion retention test starts from a seeded native founding, joins a
participant and takes an actual deliberative body. It asserts a Character with
cell `[-7, 2, 13]` and that participant as owner, saves the command log to JSON,
loads/replays it, and verifies exact metadata and state hash. The existing body,
native place, participant/controller and control relations stay unchanged.
The new authored noun remains placeless, without a body or patch. Duplicate
assertion leaves the state hash unchanged.

Character owns format-aware serialization. Absent metadata retains the former
five-field JSON representation and digest. Current postcard encodes a sixth
Option field for both Some and None so current wire values round-trip; owner
None also round-trips. Native witness-only projections preserve the old
five-field Character postcard preimage when table metadata is absent and hash
metadata when it is present. Generic hashing, witness versions and normal save
integrity checks remain unchanged.

The four preimage controls independently encode the former Character struct,
then assemble the former entity/population/applied-record bytes. They cover
native population and per-entity witness entries, nested storylet and forced
pack entries, present cell/owner hash changes, unchanged optional entity fields
and other assertion variants, and a constructed former-Character v3 archive
with retained replay/integrity validation in both execution modes. Projection
destructuring is exhaustive and preserves source field order and omissions.
Native Part remains unprojected; Q8 owns its separate loss-record controls.

The concrete hashing owner is Mere `state-witness` at pinned revision
`4d8bd70`, `src/lib.rs:40-41`: `hash_value` uses postcard serialization with
the FNV flavor. Native `digest` in `isocosm/src/lib.rs:67-71` instead hashes
JSON with SHA-256. JSON compatibility alone therefore cannot certify witness
compatibility. The native projections route only the population, cohort entity,
asserted records, storylet and pack witness sites.

`state_hash_v2` still serializes the whole native State postcard form.
Production v1/v2 archives predate authored Character; their existing fixture
checks do not certify arbitrary synthetic v2 Character archives. Historical
postcard-frame compatibility is not claimed. Q6 owns the required protocol/ALPN
bump for the changed native wire vocabulary. Consumer replication and the
compiled discarded-metadata control are still required before Q2 completion.

## Frozen source SHA-256

Hashes are of the exact working-tree bytes at 22:34:59 UTC after the passing
rerun and final compile. Git normalizes line endings when staging; these hashes
identify the tested Windows files, rather than implying archive wire identity.

| Native source path | SHA-256 |
| --- | --- |
| `shared/isocosm/src/asserted.rs` | `DE4FF83F342CDB6A7A2B98D8C45E66E1A40F60E8E5A3AADE389C9B5F34C325B5` |
| `shared/isocosm/src/asserted/apply.rs` | `032FD993D023B8CB14F09FDF6CBBBBC14FB4F0AF1A380FA178D25961C9A03FC7` |
| `shared/isocosm/src/asserted/character.rs` | `BB6CBBBD13BD9673C5E3E132B7C6053D25F31DCEB6982C2E44F34EBBF10E839E` |
| `shared/isocosm/src/asserted/fold.rs` | `1CAD30E9F61EAFE165FFF2A1734BC7C89BF7BBC22853AA4884645AC3228FB74E` |
| `shared/isocosm/src/asserted/held.rs` | `0D97E3C28E4B81041F1E6DBF61F5D1DF0EF0475A426482488CBE83975C7E7CE8` |
| `shared/isocosm/src/asserted/tests.rs` | `E0E5768ACC28CF5DAFADDF6D84601134EA87CE84D60BC26D0ED53998230CD315` |
| `shared/isocosm/src/execute.rs` | `9B6EB94017C4A16FCF4B8C01138F0AC105A4368C19581BCF00E4137AC82C3CF1` |
| `shared/isocosm/src/legacy/campaign/chronicle.rs` | `A1CC7AF3CAE8AC2B7CA31831EB732DD3CC15B0F4749D3C8DBDB1EDB9D74BA519` |
| `shared/isocosm/src/legacy/campaign/faction.rs` | `21A53F28E68C8BFF3701C17BF7CB92AC4E780EE95165C760745A7F32D9BD12DB` |
| `shared/isocosm/src/legacy/campaign/world/tests.rs` | `C4FF4CFCA442FE28FBA513856C18AAA51EB5B4FD3237B3382E695162C84E1F3C` |
| `shared/isocosm/src/schema.rs` | `E830C131B307C1150F733C37184B75218141973CF5386797DA4509C0497C62FA` |
| `shared/isocosm/src/schema/event_tests.rs` | `E03636E02B9C6627185736228A2357F0AA42DDE7FAE1E1D8C4C6E387647BF3F8` |
| `shared/isocosm/src/simulation.rs` | `96880306652B9F426FDEB58821460DCAACD125E5BF94A4D2FFC6D6C4D82BA3D1` |
| `shared/isocosm/src/simulation/preimage.rs` | `AB79026902F70BDAAA53C6B73002A3DA6BD285B0B605AC67F79D214635FFACBB` |
| `shared/isocosm/src/simulation/preimage/groups.rs` | `045E04E8BAB1D70371BCE4FB89B232316CD0AA8F00AA15DEE7A3EFCDA6B0C2CB` |
| `shared/isocosm/src/simulation/preimage/tests.rs` | `18B16854BFBBC5D67C80FD8D36AA1D76B363712F75173681B5E45A73C5D30627` |
| `shared/isocosm/src/simulation/witness.rs` | `AC847E178E1A76A32B95858A8D9BF755F3AB83727C27D578243B259DAF2BEA13` |
| `shared/isocosm/src/social/asks.rs` | `264431730D1C5BE0D45A122705AFC124870C3F911C63FE89189F0D8E5A95EEB0` |
| `shared/isocosm/src/social/deed.rs` | `51537EE8A528513808D5BDDE1BC5F807F8DD056BCC03F98975A7FBF3A6C4FB38` |

The 11 new checkpoint paths are `asserted.rs`, `asserted/character.rs`,
`asserted/tests.rs`, `legacy/campaign/{chronicle.rs,faction.rs,world/tests.rs}`,
`simulation.rs`, `simulation/witness.rs`, `simulation/preimage.rs` and
`simulation/preimage/{groups.rs,tests.rs}`, all under `shared/isocosm/src/`.
