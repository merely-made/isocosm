# Scene composition receipts

## 2026-11-04 — initial preservation and host check

Evidence root: `Code/output/mesocosm-scene-composition/2026-11-04/`.
`initial/source-manifest.json` and `initial/source.patch` retain the initial
dirty checkout over `775277e9e700dd0aaaf53205c499b417da0b303e`, including
the completed protected-root and food-contract work. The source manifest
hashes 1,588 inputs across Mesocosm, Cosmograph-host, shared crates and root
Cargo configuration. Prior frozen world outputs are untouched.

The guarded initial command uses stable lane `mesocosm-scene-composition`,
Rust 1.99.1 and `check --locked --offline -p mesocosm-genet --lib -j 4`
from `mesocosm/`. It exits 101 because the existing lockfile requires an
update, before compiling code. Full log: `initial-host-check.log`.
An isolated validation checkout will retain the original lock and inspect
the actual resolution difference without changing canonical supplier pins.

## 2026-11-04 — isolated fixed-SDK qualification

The validation worktree is `Code/worktrees/isocosm-scene-composition-validation`,
branch `scene-composition-validation`, over the same `775277e9` base and
preserved dirty source. The active product lock is ignored, so the first
bare-worktree metadata attempt lacked it. `initial/active-lock-manifest.json`
and `initial/active-locks/` retain the actual initial locks.

An unlocked/offline proposal introduced an uncoordinated cached Genet head;
that proposal is retained as `uncoordinated-proposed-host.lock` and was not
adopted. Qualification instead uses the exact Genet package/source mapping
in the already selected Mere `5431b896c8c277c4d8ce2cf33cdfd33692e82cd8`
SDK lock: core packages at `b7c0bef04613d854367d407b91c87a38b9b03b35`, and
the selected document/runtime packages at
`c0144fe59dea449519cede190bed8662f661fc22`. The selected Eidetic revision
remains `613799afd001200792e6c37d4406807848dadabc`; Netrender remains
`11e208b84a981993df8077bb6025f8b3adbe2291`.

`validation-cohort.json` records this mapping;
`sdk-host-metadata.log`/`.json` record successful locked/offline resolution.
Only the validation worktree lock changed. The canonical ignored product
lock and supplier manifests remain untouched. This is fixed-SDK evidence,
not qualification of the canonical checkout's stale lock.

## 2026-11-04 — genuine before-fix controls

All three controls execute against the initial source, before the native
revision, scene cache or presentation synchronization repairs are copied in:

- `scene-cache-prefail.log`: one executed control fails because an unrelated
  opaque section write allocates a new native body vector despite equal body
  data. Exact filter: `app::scene_cache_tests::opaque_section_publication_keeps_scene_body_storage`.
- `native-authority-prefail.log`: one executed control fails because a
  life-history-only transaction keeps `WorldTime` instead of the requested
  `Ticks`. Exact filter: `world::authority_tests::a_native_only_life_history_change_is_published`.
- `product-adoption-prefail-physical.log`: one executed control fails because
  direct presentation refresh retains `None` sight after another door
  publishes a valid `SightConfig` for physical observer 6. The fixture uses
  seed `16003851315348448882`, population 60 and actual generated placement.
  Exact filter: `app::assembly::tests::a_direct_camera_refresh_adopts_another_endpoints_relevant_section_first`.

The earlier `product-adoption-prefail.log` stops at fixture setup because
the coarse bench has no physical binding; it is not a regression result.
The corrected physical run reaches the intended stale-reading assertion.
Executed before-fix binaries are APFS-cloned in `prefix-binaries/` with
verified SHA-256 sidecars.

## 2026-11-04 — source and native-store gates

`native-store-validation.log` records six release, locked/offline tests
passing under stable lane `cosmograph-policy`. A later fingerprint audit
identifies this initial run as Homebrew Rust 1.99.0 despite its outer
`rustup run 1.99.1` prefix; the pinned final rerun is recorded below. They cover
identity-ordered stores/joins, removals and reinsertion, clone isolation,
non-Clone payloads and unchanged native save/hash/history/load/fork replay.

`integrated/validation-sync.json` hashes 109 mirrored Rust inputs and both
the validation and original product locks. The first integration no-run
launch reused an old binary because `copy2` retained source mtimes older
than its compile. That launch is unqualified. After checking all source
hashes, validation-copy mtimes were advanced and a new build launched as
`integrated-host-current-build.log`; `integrated/freshness-correction.json`
records the correction. No test result from the stale launch is counted.

`responsa-regen.log` and `responsa-check.log` record regeneration and a
successful current-manifest, doctrine-budget and declared-links check.

## 2026-11-04 — current-source correctness and device gates

The frozen current Mesocosm test binary and SHA-256 are recorded in
`integrated/test-binary.json`; `integrated-test-list.txt` explicitly contains
the new component, source-lifetime and malformed-section controls. The
post-freshness build exits 0 and compiles the changed native/runtime/view
packages. `integrated-unit-tests.log` records **73 passed, 5 deliberately
ignored, no failures**, in 201.73 seconds. The old-source cache and external
sight-adoption controls now pass in that complete run.

`native-revision-tests.log` records **20 passed, no failures**, filter
`world::`, including native metadata, commands/time/branches, same-owner and
fresh-owner tokens, cached cuts, no-op/refused/mixed products and verified
replica publication. `integrated/native-test-binary.json` binds the frozen
test binary and its hash. The original life-history control now passes.

`native-gpu-control.log` records the explicitly invoked ignored actual-device
viewport control passing: **3,072 pick samples**, 473 body hits, 932 terrain
hits and 1,667 empty hits. It compares the borrowed producer with the original
Section, checks retained callbacks add no draw/upload work, accepts current
action revision 8 on the unchanged image, rejects an unpresented changed
picture, and preserves cumulative producer work across suspend/retire and
source recreation. Its 10.63-second duration and printed CPU comparison
time are not isolated performance or physical-display evidence.

## 2026-11-04 — first release representation and live-fixture diagnostic

`release-host-build.log` records the optimized all-targets build completing
in 12m49s. `release-binaries.json` records frozen executable/test hashes and
the qualified source/validation-lock hashes. The new backend is developer
only; `component-cli-control.json` records refusal without `--dev`.

`component-cpu-comparison.log`/`.json` records the release diagnostic passing:
three actual native body-row counts, 3/12/48, cohorts of eight, five warmups
and 51 alternating-order samples per count. Native extraction, terrain and
skin construction, assertions, serialization, GPU work and result destruction
are outside the timers. Median capture+materialization costs are
52,105/248,676/1,077,626 ns; the owned-vector clone controls are
25,764/122,085/499,179 ns. Raw samples and nearest-rank p95 are retained.
Metadata traversal is a different operation and must not be compared with
capture as if it established a generic ECS speedup or regression.

The first real Assembly run uses the same binary, seed 9, population 256,
1280x800, voxel bodies and epoch 1000. `retained-command.json` binds the
invocation; `retained-host.log`, `retained.json`, `.timing.json`, `.world.json`
and captures preserve its **failed fixture**. Before scripted pause the
clock reaches tick 26 and a checkpoint; the final world has 167 living
members but no presented voxel bodies. Selection and manual-step assertions
therefore fail. These phase timings are not a successful matched comparison.

The correction is an explicit `--dev --paused` starting clock control,
implemented before window warmup. The scenario now asserts zero ticks and
steps before collecting samples; a focused control advances the host's
wall-clock origin by ten seconds while verifying no native step or saved
native change. `start-paused-sync.json` records the five refined inputs.
The original release binary, measurements, failed run and source snapshot
are retained; the correction does not retune ecological rules.

## 2026-11-04 — final pinned release and matched host evidence

The first paused rebuild selected Homebrew Cargo/rustc 1.99.0 through the
launch environment. Dependency compiler fingerprints differ while source,
features, profiles and configuration agree; the earlier 1.99.1 cache remains.
The task's misconfigured process group 1364 was interrupted after verifying
its launcher, lane, checkout and group membership. This was cancellation of
a wrong-compiler invocation, not storage cleanup or admission bypass. No
source or compiler cache was removed. `compiler-selection-correction.json`
records the scope; `release-host-paused-build.log` is unqualified.

The corrected guarded invocation pins absolute 1.99.1 PATH, RUSTC, RUSTDOC
and RUSTUP_TOOLCHAIN after Python startup, retaining the same lane, checkout
and SDK lock. `pinned-compiler-versions.json` records Cargo/rustc 1.99.1.
`release-host-paused-pinned-build.log` succeeds in 2m 20s and reuses dependencies.
`release-binaries-v2.json` records the frozen executable/test hashes,
compiler fingerprint `5614879824171296209`, final source-manifest and lock
hashes. `qualified-source-v2/` and `qualified-source-manifest-v2.json` retain
118 inputs, including the explicit paused-start refinement.

`paused-release-controls.log` passes all 8 focused Assembly controls.
The final pinned complete library run in `final-release-unit-tests.log`
passes **74 tests, 5 deliberately ignored, no failures**, in 14.24 s.
`final-native-gpu-control.log` passes the explicitly invoked actual-device
control again, with the same 3,072/473/932/1,667 total/body/terrain/empty pick
counts. The earlier 20-test native-world suite's compiler fingerprint also
matches 1.99.1; those frozen files are retained.

Both `retained-paused.json` and `components-paused.json` have `ok:true`,
no errors, and all six phases complete. Their command/exit/log files bind
the same frozen executable, source manifest, 1280x800 logical surface
(2560x1600 capture), seed 9, population 256, voxel renderer, epoch 1000 and
explicit dev-paused start. Only the backend flag differs. Runs take 349/350
host frames; phase sample counts match. Both finish at tick 3 after exactly
3 manual native steps, hash `2f1d926b35f3314f`, 263 living members, 4 scene
extractions, 17 scene draws, selected organism 38/part 1, and 0 projection
failures. Their complete saved-world files are **byte-identical**.
This is the real coarse native bench: physical-members/native-poses are 0;
it is not a physical ecology, gait or long-horizon persistence fixture.

`mixed-comparison.json` retains raw-stage percentile analysis and missing
metrics; `mixed-summary.json` adds means, counts and source comparisons.
The following are eligible host frame wall times in ms, expressed as
median / nearest-rank p95 / p99. Capture and invalid frames are excluded;
steady-after-first summaries are also retained. P99 is often the maximum
for these short phases. Stages overlap and must not be summed.

| Phase | Eligible frames each | Retained median / p95 / p99 | Components median / p95 / p99 |
| --- | ---: | ---: | ---: |
| `paused_picture` | 62 | 25.224 / 25.959 / 26.512 | 25.386 / 26.201 / 26.616 |
| `inspected_picture` | 62 | 24.703 / 25.173 / 25.688 | 24.651 / 24.954 / 24.996 |
| `camera` | 21 | 24.999 / 34.111 / 34.694 | 24.860 / 33.874 / 34.243 |
| `inspector_interaction` | 40 | 25.338 / 38.389 / 38.725 | 25.180 / 37.419 / 38.362 |
| `native_steps` | 19 | 28.322 / 56.120 / 56.120 | 28.078 / 55.647 / 55.647 |
| `post_step_picture` | 62 | 24.867 / 25.910 / 26.334 | 24.853 / 25.563 / 25.695 |

Every one of 266 eligible frames in each run exceeds both 16.7 ms and the
exact 60 Hz CPU wall-time thresholds. These are **CPU-budget exceedances**,
not measured physical refresh deadlines. Paused frames have 0 scene draws,
0 added qualified body byte work and 0 layout mutations/rebuilds. The outer
raster call's paused median is about 21.7 ms; the inner renderer timing is
about 1.4 ms. Its work/synchronization/GPU attribution remains open.

Both backends submit the same 17 scene draws, 71 mesh builds and 6,082,764
**estimated** body-mesh bytes. Tenant instance/frame bytes remain null,
not measured zero. Terrain diagnostics agree on 8,483,840 brick-upload
bytes, 48,240 uniform-upload bytes, 1 map rebuild, 5 residency changes and
17 trace passes. CPU preparation duration varies; counters are qualified
by their actual source. Paused, inspected and post-step retained phases
add no scene draw or upload work. Live phase counts include camera changes,
part selection, true inspector mount/unmount and three native steps.

`component-cpu-comparison-v2.log`/`.json` repeats the release representation
diagnostic using the final pinned test binary, five warmups and 51 samples
for 3/12/48 native body rows. Median owned-vector clone costs are
24,473/122,798/503,155 ns. Median capture+join+owned materialization costs are
49,143/252,568/1,083,960 ns; corresponding p95 costs are
52,389/291,249/1,470,452 ns. The timed operation excludes native extraction,
terrain/skin generation, assertions, destruction, output and GPU work.
It establishes conversion overhead here, not an archetype/dense-store,
incremental update or selective-query result. Retained rows stay default.

The matched captures are retained under each `*-mixed-inspection.png`.
`capture-comparison.json` records differing PNG byte hashes. Lossless RGBA8
decoding with PNG chunk CRC checks in `capture-pixels-comparison.json`
finds exactly 2 differing pixels out of 4,096,000, at DOM text positions
outside the compared scene region; that region has 0 differences. Full
bit-identical captures are not claimed. Visual inspection confirms the
native selected part accent and the inspector's organism 38/part 1 reading.

`environment.json` records Intel Xeon W-2140B, x86_64, macOS 15.8.1 and the
missing `xctrace` utility. Heap allocation totals, input-to-present latency,
physical display deadlines, GPU duration and complete tenant upload bytes
remain unavailable. The analyzer's `budget_exceeded` storage flag is not a
frame-time verdict. Source/executable bindings are in external command and
freeze manifests; the native app receipt does not embed those identities.
No cross-platform, embodied ecology or physical 60 Hz certification follows.


`final-native-store-validation.log` requalifies all **6 native-store tests**
under the explicitly pinned Rust 1.99.1 compiler and the final locked SDK
workspace; no failures, 0.02 s test duration. The initial Homebrew run remains
as earlier evidence. The final full-library, native-world, native-store and
GPU controls are all tied to their actual frozen/compiler receipts; counts
are separate suites and are not added into a unique-test total.

## 2026-11-05 — R8 current-cohort native receiving

R8 receiving validation passes on the isolated Isocosm port over published
main `86d07fab` with the published M/G/N/P chain. All 298 native tests and
the VTT, Eponym, Wing and Parry compatibility builds pass. Four C4 runs use
the same current executable, all four native replays agree and all eight
decoded scene crops match a fresh oracle from this cohort. Remote main
adoption is tracked in the separate R8 publication receipt.

### Current-cohort native and runtime result

The exact published supplier family is M `ad6aaf60ef7fcf869eac6eb313f40064029fded9`,
G `ceb5a9795a8f26ffe3c6fba25a55bfce3dd0a15d`,
N `f8811186cd82b0d4b6758b15c6bcecd415273d52` and
P `dfb9b53b315561887761deff319afed08cc33ff0`. The isolated receiving base
is published Isocosm `86d07faba63194e18c035f562eb57958d65fd05d`; its
foreign source family is preserved. The initial reviewed alignment covers
17 manifests and nine locks. Actual resolver normalization is retained
separately for the runtime cut and the later consumer checks.

| Executed native prerequisite | Result |
| --- | --- |
| Cosmograph-host full library | 69 passed |
| Mesocosm default library | 87 passed; six ignored in this default invocation |
| Explicit Mesocosm staged GPU control | One passed; 92 filtered |
| Shared implicit-body GPU integration | One passed |
| Native WorldSave integration | One passed |
| Cosmograph native store integration | Six passed |
| Isometer full library, including GPU controls | 83 passed |
| Isomere host library | 50 passed |
| Distinct executed native prerequisite aggregate | **298 passed, zero failed** |
| VTT compatibility build | `isocosm-genet` and `isocosm-native`, all targets passed |
| Eponym compatibility build | `eponym-client`, all features and all targets passed |
| Wing compatibility build | `wing-integration`, all targets passed |
| Parry compatibility build | `parry-ground` binary passed |

All four compatibility builds use locked, offline release checks. VTT and
Eponym exercise their existing consumer paths. Explicit staging for these
products belongs to R4. The first Parry check required a lockfile refresh.
Its final input aligns `hagiograph` and `impresa` with the Eidetic revision
already declared by Cosmograph at `86d07fab`. The reviewed transition is
recorded in
`isocosm-current-qualification-01/gates/parry-metadata-probe-01/post-runtime-lock-adoption.json`.

The first current-main product build exposed an omitted source prerequisite.
Restoring the exact 14-line `Runtime::synchronize_view` method in
`mesocosm/crates/mesocosm-runtime/src/runtime/world_save.rs` makes the whole
file byte-identical to its qualified R7 source, SHA-256
`307c41216400a4b15deb93323a360f6ed1ad9fa1eb9190380825a36bb8388ff0`.
The method adopts a valid accepted product section before presentation;
it does not advance or publish the world. The genuine missing-method
compile failure is retained, followed by the existing adoption/refusal,
save/replay and GPU controls. This adds one prerequisite file to the
original 60-path port; it changes no ecology rates or traits.

The runtime binary is the regular release application, with compilation
profile `opt_level=3`, `test=false`, SHA-256
`6e4e1534f65302e5258a56e06badd8517c5c790a3d02d47bd95757741f1aa487`.
The protocol runs retained/components/components/retained on that same
current executable. Each headed run exits successfully and its saved world
replays successfully. All four saves have tick 3, native state hash
`3395030683271508303`, and identical 104,498 bytes, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`.
The oracle was captured from this cohort; historical R7 identity was not
assumed as the premise of the check.

A separate fresh layout/DPI probe establishes the 2560×1600 image and
scene region `[24,120,2004,1500)`. The mixed and final decoded scene crops
from all four runs are exact, RGBA SHA-256
`cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
The scope is recorded scene pixels and native save/replay correctness.

The runtime freeze hashes 1,404 files across 19 packages, the 13 root
locks and configuration inputs, and the 61 source/index identities. The
post-run checks verify those hashes. Later Eponym, Wing and Parry checks
bind their own input generations and resolver receipts.

This correctness matrix ran with recorded external compiler and desktop
contention. The GPU timing request returned `device_unsupported` in all
267 observed unique batch reports per run. Adapter support and device
enablement were both false. GPU duration, physical refresh, input-to-display
latency, OS accessibility, platform breadth and ecological persistence
require separate evidence. C5 Didgeridoo remains conditional.

Evidence is retained under
`Code/output/mesocosm-scene-composition/2026-11-04/producer-preparation/integration/`:
`isocosm-current-qualification-01/native-runtime-handoff.json`,
`isocosm-runtime-view-sync-01/handoff.json`,
`isocosm-native-runtime-admission-01/native-admission.json` and
`actual-main-native-c4-01/runtime-handoff.json` with its `comparison.json`,
four replay results, source/lock postchecks and independent raw-save check.
The runtime handoff identifies the earlier native-product cut.
`isocosm-current-qualification-01/qualification.json` binds the complete
receiving result, including the VTT, Eponym, Wing and Parry build receipts.
The final locked Parry check is recorded at
`isocosm-current-qualification-01/gates/parry-live-check-02/result.json`.
The R8 publication receipt tracks the remote commit separately.

## 2026-11-06 — R9 source intake

The isolated query-sharing worktree starts at Isocosm
`26306b028c7c41f736c56dfc12c68d28e8a01b68`, matching the fetched
`origin/main`. R8 receiving is in its ancestry at `da56ce39`. The C4
scenario and borrowed Section producer match R8. The scenario SHA-256 is
`fbc18d99507d154a91b40d0131a78e045d5540eb801f7e7bce92790188f82a05`.

The base Mesocosm lock SHA-256 is
`2be642c47fcc768a6b5a2326e7ca2bc0e8f5070425aaa4b6724e7316bbd08812`.
The Isometer lock SHA-256 is
`5457fdfd14d21b72b7646e8b0eb95f293055ae9dd55065b56d21c19f2886edb0`.
Both lock Mere `e9ac10dc`, Genet `7095acc5`, Netrender `f8811186` and
Vello `dfb9b53b`. The current Mere revision contains Mandrake's product
work transport and Modulus's dense vector and key-map clone.

The checked terrain adapters are `GroundTerrain`, `HostTerrain` and
`PagedTerrain`, plus the palette tests' scaled wrapper. Mesocosm uses
paged projection. The VTT uses paged projection and a native ground path.
Eponym uses host map replacement and native ground. This source intake
sets the receiving check scope. Fresh executable and runtime results
belong to the subsequent R9 qualification cut.

## 2026-11-06 — R9 query-sharing qualification

The locked release builds use the same `26306b02` base and 492 matched
compiler package identities. The runner verifies the source manifests
before and after the four arms. A's executable SHA-256 is
`58b8032ddd53015d92307eec1a2ca5a172271139c822a3d4406be328f22ba065`, and B's is
`2abca973de2074f46eb4c13280544ce1cab9933a9b84d749e3515645e71c470e`. Their source manifest SHA-256
values are `4b0bedde20bb4532b8fb95cf587d4203563738c0fbe48bfae83fbcf130f9d68e` for A and
`cb5efd74e529b252b61a0925f3712868b67fa9e038314f70c77f0f7b0912baf1` for B.

The candidate records **238 passing tests**. Isometer passes 88, the
Mesocosm default library passes 87, and the staged, picking, implicit-body
and WorldSave controls each pass one. Standalone Lens records 59 passes,
two failures and two ignored tests. The full clean-baseline Lens control
reproduces both failures and their diagnostics with identical compiler
feature contexts. The failures are the level-section channel ladder and
the unlit floor's dither assertion. The explicit GPU refresh control
writes a dirty brick, refuses that prepare and verifies a full upload on
retry with the same texture allocation. The held pick keeps its old
material. Locked release checks pass for all targets of `isocosm-genet`
and `isocosm-native`, and for all features and targets of `eponym-client`.
The VTT audit matches 689 package source identities and finds identical
package trees for five Dramatis entries with differing commit labels.
Eponym matches all 483 package source identities.

Each arm runs the same paused C4 fixture, with one viewport producer,
ten voxel bodies and zero physical bodies. All four headed runs and
four native replays pass. The four 104,498-byte saves match at tick 3
and state hash `3395030683271508303`, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`. Eight decoded RGBA crops match,
SHA-256 `cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
The crop uses declared bounds `[24,120,2004,1500)` from the prior layout
probe. The current images are 2560×1600 and producer extent is 1980×1380.
The matching pixels cover those coordinates. Both retained full images
match their counterparts. In the components pair, both full captures
differ by one level in one RGB channel at `(273,40)`, outside the crop.

Each backend makes 13 fresh timed terrain captures and 253 retained
snapshot clones. B copies four maps in the camera phase and shares six
maps in inspector interaction and three in native steps. Timed copied
bytes fall from 109,085,184 to 33,564,672 per backend. Across all 18
lifetime captures, including untimed work, the count changes from 18
copies to five copies and 13 shares, and copied bytes fall from
151,041,024 to 41,955,840. Both arms reference the same dense payload
totals. Counts cover pointer and atlas bytes. The source review locates
the only new snapshot materialization at capture. Refresh writes or
replaces the render-owned map and updates its witness.

| Timed phase measure | Retained A → B | Components A → B |
| --- | --- | --- |
| Mean capture CPU, ms across 13 calls | 3.950 → 1.546 | 2.931 → 1.017 |
| Mean draw CPU, ms across 13 calls | 6.584 → 6.555 | 6.496 → 6.547 |
| Draw plus capture CPU sum, ms | 136.937 → 105.308 | 122.557 → 98.331 |
| Paired UI median / p95 / p99, ms | 16.166 / 17.145 / 26.458 → 16.160 / 16.674 / 21.129 | 16.168 / 16.529 / 25.232 → 16.166 / 16.564 / 21.779 |
| Paired pictures over 16.7 ms | 14 → 13 | 12 → 12 |

Each paired UI summary covers 266 pictures and 532 profiled attempts.
Before and after process samples show no compiler or build guard. The
desktop stayed live, with one serial observation per source/backend.
The direct CPU sums cover the named scopes. Paired callback portions
and native latency have separate meanings, and physical refresh requires
separate evidence. Each run's 267 observed batch reports record
`device_unsupported` for GPU timing, leaving GPU duration unavailable.

The retained evidence lives under
`Code/output/mesocosm-scene-composition/2026-11-04/query-sharing-r9/`.
`matrix/r9-c4-01/comparison.json` has SHA-256
`85184186c1c78ecbacb8c807e12d7f47060d7187d730f503a90f22a5ff8626ed` and binds the plan,
source checks, four replays, copy accounting, native saves and crops.
`runtime/baseline-lens-tests-02-full-control/result.json` records the
matched failures. `qualification.json` binds the
candidate tests and the VTT/Eponym checks. The earlier Mesocosm test
generation differs from B by the Lens `cfg(test)` ray-fixture constructor
repair, recorded in `isometer-test-fixture-repair-01.json`. The later Lens
suite executes that fixture. Remote main adoption has its
own publication receipt.
