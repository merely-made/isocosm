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

## 2026-11-15 shared feeding searches in a bodied ecology

Seed 9, four sites, five A1 diet roles, bodies, 60% inherited variation,
limits and rot. Both arms used counted births and Individuals execution.
Windows release build with `CARGO_PROFILE_RELEASE_OPT_LEVEL=1` and
`COSMOGRAPH_PROFILE_ACTIVE=sum`. The baseline was commit `2b822962`.
Paired serial runs on a shared workstation.

| Founders | Ticks | Baseline seconds | Shared searches seconds | Grazer planning seconds, baseline to shared | Evaluations | Live members | Anatomical draws | Retained rows |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 256 | 60 | 8.972 | 5.216 | 3.720 to 0.701 | 45,054 | 1,744 | 135 | 1,814 |
| 1,024 | 36 | 31.889 | 17.479 | 15.416 to 1.980 | 62,617 | 6,518 | 149 | 4,719 |

Final state hashes matched for both pairs (`5d34379927ffaf1a` and
`4fef0fcfea8d4e11`). Matter was conserved. The anatomical draw counts
use lineage, region sizes and absence; retained rows include bodies with
different stores or histories. Lineage 2 was absent from the living
population at both endpoints. The tests exercise the native scheduler
headlessly.

Reproduce with `cargo test --release --manifest-path shared/cosmograph/Cargo.toml --test cohort_ecology -- --ignored --nocapture`.
Set `COSMOGRAPH_MEASURE_POPULATION` and `COSMOGRAPH_MEASURE_TICKS` to
the corresponding row. `COSMOGRAPH_MEASURE_SEED` defaults to 9 and
`COSMOGRAPH_MEASURE_COUNTED` defaults to 1.

## Matched body development, 2026-11-14

Guarded release probe on the Intel iMac Pro, local Cosmograph at
`3d15c593` with the existing ecology patch. Inputs, toolchain, source
diff, lockfile, executable and CSV are retained in
`Code/output/cosmograph-world-time/body-generation-2026-11-14/`.
The first fixture re-expresses a seed-9 animal recipe with a single
`Repeat` and the same tagmata. The second is the existing
`bodied::tips` preset. Each fixture asserts equality of `Soma` and
developed `Body` for all 512 seeds when given pre-unfolded tagmata or
when unfolding first. Each timing is the median of five loops.

| Fixture | Parts at seed 0 | Distinct bodies in 512 draws | Unfold | Draw and develop | Draw and read shared body |
| --- | ---: | ---: | ---: | ---: | ---: |
| Re-expressed animal | 13 | 45 | 0.49 µs | 128 µs | 1.05 µs |
| Tip-grown plant | 152 | 1 | 12.18 µs | 1,218 µs | 2.00 µs |

Tip-grown plant JSON is 284 bytes for grammar alone, 3,751 bytes for its
unfolded tagmata, and 58,341 bytes for the developed neutral body document
and parts. The one-rule animal grammar adds explicit bytes over its
already compact tagmata (364 versus 315). These are serialized component
sizes. The timed lookup draws a soma and reads an existing body by
reference. The probe covers declarative construction and reads. Birth
accounting, per-member writes, save size, ticking, physics and rendering
require their own measurements.

## Ordinary body-map founding, 2026-11-14

The [authored input](../../../../shared/cosmograph/examples/body-map.toml)
founds an A1 ecology with 30 living founders on one site through
`Founding::generate`.
It gives `lineage:0` a food-fixing frond on its root and a second run
whose `Repeat` rule grows one segment to two. With variance of one,
seed 9 draws two founders with one repeated region, six with two, and
four with three. Increasing `Repeat.by` from 1 to 2 gives each of those
founders one more region. Founder cohorts share a draw.

The control changes only the binding of `region:defence` from
`part:cyanide-leaf` to `kind:frond`. Both kinds have the same extent and
nine functional cells. A frond fixes food and a cyanide leaf secretes
toxin. The native guard rule charges rent on the secreting cells and
wounds a grazer at a bite up to the configured four-cell cap.

In each seed-9 single-bite control, both editions pass the same amount of
plant matter to the grazer.

| Repeated regions | Matter eaten in either edition | Grazer cells lost to plain frond | Grazer cells lost to cyanide leaf |
| ---: | ---: | ---: | ---: |
| 1 | 2 | 0 | 3 |
| 2 | 1 | 0 | 4 |
| 3 | 1 | 0 | 4 |

Three matched seed pairs each ran for 35 ticks in native grouped
execution with no player actions. The flow reader counted plant matter
eaten by the grazer lineage and matter the producers paid in guard rent.
The harm reader counted cells logged against grazers for toxin. The
birth check followed new parent edges and verified that each seedling
grew the same number of repeated regions as its parent with the mapped
tissue. The test checked total matter at every tick.

| Seed | Region | Plant matter eaten | Toxin wound cells logged | Guard rent matter | Plant births | Live plants at tick 35 |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 9 | Frond | 437 | 0 | 0 | 21 | 21 |
| 9 | Cyanide leaf | 170 | 406 | 139 | 1 | 0 |
| 17 | Frond | 452 | 0 | 0 | 8 | 13 |
| 17 | Cyanide leaf | 175 | 620 | 87 | 2 | 0 |
| 31 | Frond | 191 | 0 | 0 | 0 | 1 |
| 31 | Cyanide leaf | 71 | 259 | 48 | 0 | 0 |

All three toxic editions lost the producer lineage by tick 35. The
binding changes both the plant's fixing tissue and its defence, so the
population counts measure their combined effects in this fixture. The
four new tests, 57 existing integration tests and 25 existing repertoire
unit tests passed in release. Three opt-in ecology measurements were
skipped.

The native CLI was checked with two frozen release binaries. The old one
refused `--founding`. An unmapped control (`--seed 9 --population 24
--sites 2 --lineages 3 --cohort-size 2 --ecology --ticks 12`) produced
byte-identical saves with both binaries, state hash `83cfc6db0c2d68ce`.
The toxic map saved at tick 17 then reopened and advanced to tick 35
reached the same genesis, state hash (`cf2e10341d2e8b00`) and witness as
an uninterrupted 35-tick run. Go through `--founding <file> --ticks N
--output <save>` and then `--load <save> --ticks N` to reproduce. Inputs,
executables, saves and release logs are retained at
`Code/output/cosmograph-world-time/body-map-founding-2026-11-14/`.

## Gland competition within a lineage, 2026-11-15

The [optional-gland map](../../../../shared/cosmograph/examples/optional-gland.toml)
grows two fixing fronds on every plant and draws once per founder cohort
for an extra gland. The existing development law omits eligible borne
parts and keeps fixing appendages. Founding records each mapped cohort's
draw so that provisioned births carry the parent's form. Previous body
maps retain zero absence odds by default.

The [native assay](../../../../shared/cosmograph/tests/body_map_competition.rs)
ran both forms together for 35 ticks on one A1 site at seeds 9, 17, 31
and 47. It varied the grazer's feeding period and the gland's upkeep.
Each setting started across its four worlds with 22 plain and 36
gland-bearing plants. The table pools live plants at tick 35. The last
column counts seeds where glands gained a share of the live plant
population relative to their own founding share.

| Grazer feeding | Matter per gland upkeep | Plain alive | Gland alive | Seeds with a gland share gain |
| --- | ---: | ---: | ---: | ---: |
| Off | 1 | 180 | 192 | 0 of 4 |
| Off | 2 | 218 | 152 | 0 of 4 |
| Off | 3 | 260 | 80 | 0 of 4 |
| Every third tick | 1 | 38 | 28 | 0 of 4 |
| Every third tick | 2 | 44 | 15 | 0 of 4 |
| Every third tick | 3 | 25 | 1 | 0 of 4 |
| Every tick | 1 | 9 | 3 | 1 of 4 |
| Every tick | 2 | 9 | 1 | 1 of 4 |
| Every tick | 3 | 14 | 0 | 0 of 4 |

The two share gains under grazing every tick were seed 9 at one matter
(one plant of each form survived) and seed 17 at two matter (one gland
carrier survived). Gland carriers recorded zero births across all
12 worlds with grazing every tick. Under the default three-tick period
and two-matter rent, the native records show 37 plain births and 21
gland-bearing births. These runs measure short-term competition with
the current native meal and upkeep rules at one site.

The previous CLI refused the new input field. Release tests checked
the two extreme odds, rejection of an invalid denominator, mixed
founders, anatomy and inheritance, save/reopen, and conservation at
every measured tick. All 11 selected tests passed. The current CLI saved
the seed-9 mixed world at tick 17, reopened it, and reached the same
tick-35 genesis, state hash (`3a208d018da6c730`), and witness as the
uninterrupted run. Its original body-map control was byte-identical to
the previous release save. The input, CSV, saves, release binary and
logs are retained in
`Code/output/cosmograph-world-time/gland-competition-2026-11-15/`.
