# Lineage rounds receipts

## 2026-11-14 inherited variation and anatomical sharing

Windows, Cosmograph release profile with `CARGO_PROFILE_RELEASE_OPT_LEVEL=1`.
`cargo test --manifest-path shared/cosmograph/Cargo.toml --release --lib measure_shared_birth_anatomy -- --ignored --nocapture`.
Seed 5, 60% variation, fixed founding family, no terrain or renderer. Timed
the same child growth requests with and without the cache. The numbers include
parent cloning and collecting distinct forms. They exclude provisioning,
scheduler work, and display. One run per arm on a shared machine.

| Founding population | Body builds requested | Distinct forms | Uncached ms | Cached ms |
| ---: | ---: | ---: | ---: | ---: |
| 256 | 192 | 100 | 45.83 | 39.53 |
| 2,048 | 1,696 | 446 | 559.47 | 543.92 |
| 16,384 | 13,408 | 588 | 3,829.56 | 3,376.66 |

The birth test reproduced the state hash across Individuals and Grouped
replay, with total matter unchanged. The fold test retained one record
holder and two different counted forms. It compacted 11,001 members into
three rows and kept the same totals per account. The fold test exercises
state compaction and leaves the cost of advancing that population unmeasured.

## 2026-11-14 counted far births

Windows, Cosmograph release profile with `CARGO_PROFILE_RELEASE_OPT_LEVEL=1`.
`cargo test --manifest-path shared/cosmograph/Cargo.toml --release --lib schedule::births::tests -- --nocapture`.
Compared both founding settings at seed 9 in Individuals mode. One site,
one lineage, 600 well-fed unbodied parents and the generated A1 birth
process on a one-tick period. The fixture runs just that process.
Median of five runs per arm on a shared machine.

| Birth record | Newborns | Population rows after the tick | Events | Notes | Family links | Tick ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Individual | 600 | 1,201 | 600 | 600 | 1,200 | 34.41 |
| Counted | 600 | 3 | 0 | 0 | 0 | 25.81 |

Both settings retained the same matter by account and the same total
reproduction weight for drift. Counted birth tests cover rollback,
replay through save/load, equality across advance sizes, splitting and
merging counts, and direct and record-holder family identity.
`cargo check --offline --manifest-path mesocosm/Cargo.toml -p mesocosm-genet`
passed. In a new generated A1 world, enable the path with
`--cohort-births --population-band 1`. The counted path folds identical
body states after birth ticks. Birth evaluation and development still run
per parent and newborn. The fixture leaves multi-lineage ecology,
corpse retention, and headed play performance unmeasured.
