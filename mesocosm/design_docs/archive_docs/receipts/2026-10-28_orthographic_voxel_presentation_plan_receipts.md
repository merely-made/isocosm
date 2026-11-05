# Orthographic presentation receipts

## Token representation comparison, 2026-11-03

Authorized comparison: one existing token recipe, live miniature and cached
sprite, actual captures and measured costs. This is an isolated renderer
probe, not a headed VTT session, L6 completion, or full L10 qualification.

### Source and retained evidence

- Example: `shared/isometer/crates/isometer-render/examples/token_compare.rs`;
  helpers, shader and viewer in its `token_compare/` directory.
- Source checkout observed at start: `33ea4199`; shared working tree, with
  pre-existing dependency and network work preserved. The workspace lockfile
  resolved the supplier revisions already in its modified manifest, plus the
  example's `serde_json` development dependency.
- Raw evidence: `Code/testing/isocosm/token-comparison/2026-11-03/`:
  `recipe.json`, `comparison.json`, four native-size pairs, four enlarged
  pairs, three crowded-board pairs, and viewer verification captures.
- Recipe: existing `bake::demo::hero`, one layer, 10 x 24 x 8 voxels, one
  rigid part, 67 greedy quads / 402 vertices. This is pipeline art, not the
  denizen generator or an animated Mesocosm organism.
- Adapter: NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan, driver 610.88.
- Retained-output cap: 100 MB across this comparison's runs and viewer checks.

### Method

The live arm uses the existing `LiveBodyRenderer`, its immutable volume
cache, palette table and instanced draw. The experimental sprite arm captures
that same renderer once at four quarter-turn facings into 64 x 64 RGBA8
colour and Depth32Float layers. A six-vertex instanced quad reads nearest
colour and per-pixel depth. Pose, lighting and projection are fixed; moving
tests translate instances and select among the four captured facings.
This isolates caching from differences in the existing CPU baker's shading
and projection. It does not establish CPU-baker pixel parity.

Release build, 1024 x 768 target, 24 warm-up frames per arm and round,
96 measured frames per round, three rounds (288 samples per result),
alternating arm order. GPU timestamps surround the common clear and the
draw; CPU wall time includes common input preparation, encoding and queue
submission, excluding the final completion wait. Readback, PNG encoding,
device and pipeline creation are outside frame timings. Other Cargo jobs
were active during the task; CPU tail latency is not an idle-machine bound.
Timings describe this adapter and probe, not an application frame budget.

### First run: costs

Static tokens, median microseconds:

| Tokens | Live CPU | Sprite CPU | Live GPU | Sprite GPU |
| --- | ---: | ---: | ---: | ---: |
| 1 | 57.7 | 42.9 | 8.192 | 8.192 |
| 64 | 68.4 | 38.1 | 19.456 | 12.288 |
| 256 | 78.1 | 42.3 | 46.080 | 22.528 |
| 1024 | 166.9 | 75.4 | 150.528 | 63.488 |

Moving tokens, median microseconds:

| Tokens | Live CPU | Sprite CPU | Live GPU | Sprite GPU |
| --- | ---: | ---: | ---: | ---: |
| 1 | 65.6 | 87.6 | 8.192 | 7.168 |
| 64 | 65.6 | 47.0 | 17.408 | 11.264 |
| 256 | 91.1 | 50.6 | 41.984 | 20.480 |
| 1024 | 206.2 | 63.5 | 135.168 | 59.392 |

All percentiles and sample counts are in `comparison.json`. For 256 static
tokens GPU p95 is 47.104 / 23.552 us; CPU p95 is 180.8 / 94.7 us. A lone
moving sprite's CPU median is worse in this run; the result is a crowd
saving, not a universal per-token win.

Initial CPU mesh construction: 0.1552 ms. The first four-facing sprite bake
cost 8.1561 ms wall time, including texture allocation, first mesh upload and
GPU completion; its four timestamped draws total 32.768 us. Device and
pipeline creation are recorded separately in the JSON. The live geometry
upload is 12,864 bytes, plus a 4,096-byte palette upload. The sprite cache is
65,536 bytes colour plus 65,536 bytes depth, shared by every copy of this
appearance. These byte counts exclude common targets, pipeline resources
and instance-buffer capacity. At 1024 moving copies instance uploads are
114,688 bytes live / 32,768 bytes sprite; unchanged frames upload neither.

### Visual checks and limits

An independent process run is retained at
`Code/testing/isocosm/token-comparison/2026-11-03-confirmation/`, from the
formatted example. Every native and board pixel comparison is again exact.
At 256 static copies GPU medians repeat at 46.080 / 22.528 us; at 1024 they
are 148.480 / 63.488 us. CPU medians at 256 are 60.9 / 38.7 us and at 1024
137.4 / 54.1 us. Initial cache wall time is 26.80 ms in this process versus
8.16 ms in the first, so neither is a universal bake-latency promise.
`provenance.json` in the first run records the binary and source hashes and
the shared-worktree boundary. Combined retained captures and checks remain
under 100 MB.

- All four native-size facing pairs match byte for byte (490 opaque pixels
  each). Board pairs at 64, 256 and 1024 copies also match byte for byte;
  the 1024-copy scene overlaps bodies and exercises per-pixel depth.
- At 4x enlargement, the live arm renders a new 256-pixel view; the sprite
  magnifies its 64-pixel cache. Both captures were examined. The live edges
  and small features remain finer; the enlarged sprite shows fixed pixels.
- Viewer controls were exercised in headless Edge. Images loaded, count and
  moving controls changed the measured rows, and the 360-pixel layout had no
  root horizontal overflow. The viewer displays captures, not a live game.
- Not covered: terrain intersections, fog, picking, animation, free camera,
  changing lighting, many distinct appearances, or native-playable bodies.
  Those need their own invalidation and integration work. One simple rigid
  recipe cannot establish a general sprite/mesh break-even point.

Reproduce from `shared/isometer` (fresh output directory required):

```powershell
cargo run --offline --locked --release -p isometer-render --example token_compare --target-dir C:/t/cargo-targets/isocosm --config 'build.build-dir="C:/t/cargo-targets/isocosm/build"' -- C:/Users/mark_/Code/testing/isocosm/token-comparison/<fresh-run>
```

Checks: native-size equality assertions in the GPU run, release compilation,
`rustfmt --check` on the example and its modules, Python syntax parsing,
`git diff --check`, and `responsa check`. The reusable existing target is
`C:/t/cargo-targets/isocosm`; no isolated target, Cargo home or worktree was
created for this comparison.

## Generated articulated creature and recurring cache refresh, 2026-11-03

Authorized follow-up: the same comparison with a generated Mesocosm creature
moving its limbs, including the cost of updating the sprite cache. This
qualifies a headless renderer experiment, not a playable Mesocosm session.

### Anatomy, motion and rendering owners

- `shared/cosmograph/examples/export_creature_motion.rs` exports native
  generated anatomy and admitted foot motion as JSON. The renderer has no
  dependency on Cosmograph or a game runtime.
- Seed `20261103`, population 60, ecological founding. Stock `Bodies` had
  no articulated leg chains at this seed. The disclosed alternate domain
  uses the existing `SpacedRoster` founding, roster enabled and generated
  proportions. The first leg-bearing cohort admitting the diagnostic +x
  step is id 38: four legs, 34 parts. No earlier eligible step failed in
  that domain; the exporter retains the attempts and the domain change.
  Anatomy, matter and strength were not edited to obtain a step.
- A flat diagnostic pad, native `Grounding`, `Walking::default()` and
  `plan_stride` supply one admitted step. Its 24 `stride_sample` frames
  supply body-local physical soles to the existing `walking_anatomy` and
  physical-chain skinning. All four feet change relative to the root;
  the result has 60 implicit surface primitives. This is actual articulated
  motion, rather than a rigid turn or an invented sinusoidal gait.
- The native plan took 5.10 ms once. Sampling 24 frames and assembling their
  JSON took the separately recorded `sampling_24_us`; neither is charged to
  the renderer's per-frame table. This does not measure ongoing simulation,
  collision admission, decisions, ecology or creature growth.
- `shared/isometer/crates/isometer-render/examples/creature_compare.rs`
  uses the existing smooth implicit renderer, with plain declared-part
  envelope volumes and fixed tint/light. It does not qualify Mesocosm's
  full appearance/terrain compositor, nor compare against an animated
  greedy-mesh implementation. The previous rigid-token costs use another
  rendering path and are not a limb-animation baseline.
- The sole reusable runtime change exposes the implicit renderer's retained
  depth texture for downstream composition and makes that depth sampleable.
  The probe's sprite pipeline reads retained colour and depth directly;
  there is no CPU image download or texture re-upload in cache refresh.

### Evidence and measurement

Checkout: `33ea4199568d4f858eb6051b53cd47b112bfd4df`, with concurrent manifest
and network work preserved. Source snapshots and SHA-256 hashes are retained
beside the final receipt in `provenance.json` and `source/`.

Evidence root: `C:/Users/mark_/Code/testing/isocosm/creature-comparison/`:

- `2026-11-03/motion.json`: generated document, generator selection, native
  endpoints, stride, fixed-point roots, physical feet and bearing sets.
- `2026-11-03-aligned/`: final 96 single-creature PNG captures, 16-creature
  live/sprite boards, `comparison.json`, preview checks and provenance.
- `2026-11-03-aligned-confirmation/`: independent process, case order reversed.
- Earlier exploratory, validated and final-labelled runs remain retained.
  The qualified results here are the later **aligned** pair. Earlier boards
  placed some bodies on half pixels; their native-cache silhouettes differed
  by 978 pixels. The aligned scene rounds screen placements to whole pixels,
  as the prior token probe did. Retained data exposes that correction.

Adapter: NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan, NVIDIA 610.88.
Each process measures three rounds of 72 frames per case after 24 warm
frames per round, at 1024 x 768 with 64-pixel creature images. CPU wall time
includes anatomical skinning, allocations, layout, buffer uploads, encoding
and submission. GPU timestamps bracket refresh (when due), image writes and
the final display draw. Completion wait, readback and PNG encoding are outside
frame timings. Other builds were active: CPU results are observations from
this shared machine. CPU and GPU times overlap and must not be added.

Cache policies: every frame, or one refresh followed by three held frames.
At a hypothetical 60 display frames/s these are 60 and 15 refreshes/s.
The native motion is replayed across 24 phases. Shared mode uses one pose
for all copies; separate mode uses one slot per creature with staggered
phases. Repeated poses among 64 copies are deliberately not deduplicated.
This measures those two cache policies, not a minimum possible cache.

Arithmetic mean microseconds per displayed frame, range across both runs:

| Copies / policy | Mode | CPU us | GPU us |
| --- | --- | ---: | ---: |
| 16, separate | Live | 669.9--759.0 | 1037.2--1043.7 |
| 16, separate | Refresh every frame + sprite | 696.8--723.9 | 1012.4--1013.3 |
| 16, separate | Refresh every fourth frame + sprite | 260.3--314.0 | 262.9--268.8 |
| 64, separate | Live | 2197.7--2847.6 | 2925.3--2938.4 |
| 64, separate | Refresh every frame + sprite | 2204.3--2552.2 | 2669.8--2696.5 |
| 64, separate | Refresh every fourth frame + sprite | 626.5--737.4 | 677.9--754.6 |
| 64, shared | Live | 2231.9--2352.8 | 2911.9--2912.8 |
| 64, shared | Refresh every frame + sprite | 285.2--322.2 | 575.0--577.4 |
| 64, shared | Refresh every fourth frame + sprite | 157.5--159.8 | 149.8--150.6 |

Means include the refresh frame. At 64 separate slots, first-run fourth-frame
refresh+draw medians are 2181.8 us CPU / 2649.1 us GPU; held-frame GPU median
is 10.24 us. At 16 separate slots they are 662.4 / 1021.95 us, versus 8.192 us
held GPU. The JSON retains sample counts, means, p50/p95 and uploads for both
refresh and held frames. A held-frame median alone would hide most of the
cost. A lone live body varies from 564 to 622 us GPU mean across the two
orders; this probe does not establish a universal per-creature break-even.

### Cache size, visual checks and limits

- Each retained 64 x 64 pose is 16 KiB RGBA8 + 16 KiB Depth32Float: 32 KiB.
  One shared slot costs 32 KiB, 16 separate slots 512 KiB, 64 slots 2 MiB.
  These are one-facing, one-lighting, current-pose caches without mipmaps,
  not pre-baked animation libraries or measured GPU memory bandwidth.
- Additional probe working storage is 794,848 bytes for the bake renderer's
  frame/primitive/draw buffers, 32,768 bytes sprite instance capacity, and
  16 KiB per slot for the flat terrain-depth input. The retained depth is
  shared with the bake renderer, without a second copy. Common display
  targets, live renderer, source geometry, pipelines and driver allocation
  overhead are additional. This is logical resource size, not a VRAM profiler.
- Initial allocation, pipeline creation and first bake are recorded in each
  case's `initial_cache_wall_us`. They follow the captures with warmed driver
  pipelines; they are not cold application startup. Recurring refresh does
  not recreate textures or pipelines.
- All 24 fully refreshed native 64-pixel captures match byte for byte.
  The aligned 16-copy board has zero silhouette differences and 11 shading
  pixel differences, consistent with floating-point differences in the
  separately transformed live draw;
  no exact arbitrary-world-position equivalence is claimed. The older
  half-pixel board makes the sampling limit visible.
- At 256 pixels the live arm renders new detail, while the sprite enlarges
  its 64-pixel image. Both were inspected. The interactive comparison shows
  actual captures, one slow non-looping step and a pose scrubber. Every-fourth
  refresh displays the corresponding held captured pose, not interpolation.
- Changes in pose, camera/facing, lighting and anatomy need invalidation.
  Only pose is varied here. Distinct appearances, free camera, terrain/fog
  intersections, picking, many creature species and product integration
  remain unqualified. Shared-pose results apply to copies sharing the full
  visual state, not arbitrary native organisms.
- All cache slots refresh together in the reduced-cadence case. Spreading
  refresh work over successive frames may alter the spikes and batching
  efficiency; that schedule was not measured.

Interpretation: full-rate separate caches save little against this live
articulated renderer. Sharing visual states or accepting coarser image
refresh produces the substantial savings. Neither changes simulation rules
or removes the need to evaluate native physical motion.

Reproduce from the repository root, with fresh output paths:

```powershell
cargo run --release --manifest-path shared/cosmograph/Cargo.toml --example export_creature_motion --target-dir C:/t/cargo-targets/isocosm -- <motion.json>
cargo build --release --manifest-path shared/isometer/Cargo.toml -p isometer-render --example creature_compare --target-dir C:/t/cargo-targets/isocosm
C:/t/cargo-targets/isocosm/release/examples/creature_compare.exe <fresh-output> <motion.json>
$env:CREATURE_COMPARE_REVERSE='1'
C:/t/cargo-targets/isocosm/release/examples/creature_compare.exe <fresh-confirmation> <motion.json>
Remove-Item Env:CREATURE_COMPARE_REVERSE
python shared/isometer/crates/isometer-render/examples/creature_compare/report.py <fresh-output> <viewer.html> <fresh-confirmation>
```

The exporter uses its fixed disclosed seed/domain; its output path must have
an existing parent. Frame output refuses to overwrite an existing comparison.
No new Cargo home, target or worktree was created.

Checks: both release examples compile and run; 24 native pairs are equal;
all four feet move relative to the root; the aligned board's silhouettes
agree. The final inline viewer loads all selected images, plays once to pose
24 and stops, scrubs native/live and held cached poses, switches cache policy
and measured rows, and fits a 360-pixel browser (328-pixel iframe) without
horizontal root overflow. Headless Edge reported no JavaScript errors.
UTF-8 labels and native 64-pixel image dimensions were checked. `rustfmt
--check`, Python syntax parsing, scoped `git diff --check` and `responsa
check` pass. Retained output is approximately 75 MB, below the 100 MB cap.
Unrelated shared work remains uncommitted and was not staged or reset.

## Live/sprite hybrid with zoom-dependent refresh, 2026-11-03

Authorized trial: live creatures up close, cached sprites farther out, with
refresh slowing as the view zooms out; compare it with the existing plans.
This is an R0/R1 renderer probe, not a product rollout or new simulation tier.

### Owners, source and settings

- Entry: `shared/isometer/crates/isometer-render/examples/hybrid_compare.rs`.
  It reuses the creature comparison's generated-body, GPU, atlas and sprite
  helpers. New code lives in `creature_compare/hybrid.rs` and its `hybrid/`
  children. No new runtime crate, device, game dependency or outcome model.
- Input is the previous `creature-comparison/2026-11-03/motion.json`: seed
  20261103, id 38, four legs, 34 native generated parts, 60 implicit surface
  primitives and the same admitted 24-pose stride. The original generator
  domain and flat-pad limitations still apply.
- Orthographic projected union bounds, not camera distance, set zoom. World
  placements remain fixed; zoom changes projected size and visibility.
  The benchmark has 64 clones with independently staggered phases. The center
  subject is examined and remains live; other visible subjects can use sprites.
  Captures use a 16-clone, 256 x 192 crop; timings use 1024 x 768. These are
  equal-size specimens under one orthographic camera, not a perspective scene
  whose depth creates mixed sizes.
- `Config` accepts an optional JSON file after the motion path. Trial values:
  enter R0 at 56 projected pixels, leave at 44; sprite periods 2, 4 and 12
  display frames, or 30/15/5 updates/s at 60 displayed frames/s. Leave period 2
  below 26 px, enter it at 30; enter period 12 at 12 px, leave above 16.
  The R0/R1 dither takes eight displayed frames. These are experiment settings,
  not adopted product defaults or ruled quality thresholds.
- The choice function explicitly takes previous choice, projected size,
  held-detail bound, examined state and configuration. Its bounded R0/R1
  checks pass: 0 rung switches inside the band; the zero-band control produces
  63 switches; an examined subject cannot exceed the held-detail bound; removing
  that bound admits the control's finer rung; rate-band oscillation is stable.
- A cache bake removes the native root offset from the local pose. The sprite
  instance applies the current native root and its clip-depth offset on every
  display frame, including frames that keep old limb pixels. The examined
  body uses current native anatomy skinning. No physical stepping, decisions,
  legal position, body document or saved-world rule changes with image cadence.
- Mixed drawing shares the live target's retained depth, then draws sprites
  into it. The switch renders both complete candidate pictures and chooses
  their colour and depth with a deterministic 4 x 4 ordered mask. The retained
  GPU verification checks 442,368 output pixels across nine mask weights:
  every pixel equals one input, zero invented pixels. A deliberately averaged
  alpha control produces 1,259 pixels absent from both inputs. This proves
  that bounded colour-choice condition, not general picking, terrain/fog
  composition or independently transitioning product subjects.

### Retained evidence and frame costs

Root: `C:/Users/mark_/Code/testing/isocosm/hybrid-comparison/`.
`2026-11-03/` holds `hybrid.json`, 288 live/mixed motion captures at six zoom
levels, eleven switch/reference captures, checked previews, source snapshots
and provenance. `2026-11-03-confirmation/` holds the independent reverse-order
run. Retained output is about 60 MB, below this lane's 100 MB cap. Earlier
creature and token evidence was not overwritten or deleted.

Observed checkout: `33ea4199568d4f858eb6051b53cd47b112bfd4df`, shared dirty
tree. Adapter: NVIDIA GeForce RTX 4060 Laptop GPU, Vulkan, NVIDIA 610.88.
Each steady case has three rounds of 72 measured frames after 24 warm frames
per round. CPU includes skinning, visibility-list use, allocations, buffer
uploads, encoding and submission. GPU includes the offscreen bake when due,
the examined live draw and current-root sprite composition. Cache refresh and
held frames both contribute to the arithmetic means. View visibility is
derived once per fixed-view case; full application scene traversal is not
timed. Native planning/physics, readback, PNG encoding and resource creation
are outside these steady frame times. Concurrent work limits CPU conclusions.

Mean milliseconds per displayed frame, ranges across the two processes:

| Projected bounds | Active trial policy | Visible | All-live CPU / GPU | Hybrid CPU / GPU |
| --- | --- | ---: | ---: | ---: |
| 10 px | 1 live, 63 sprites at 5 Hz | 64 | 2.01--2.42 / 0.59--0.65 | 0.36--0.44 / 0.50--0.54 |
| 24 px | 1 live, 63 sprites at 15 Hz | 64 | 2.01--2.28 / 1.25--1.26 | 0.68--0.75 / 1.01--1.02 |
| 40 px | 1 live, 63 sprites at 30 Hz | 64 | 2.07--2.32 / 2.47 | 1.19--1.35 / 1.68--1.69 |
| 48 px, approached from far | 1 live, 63 sprites at 30 Hz | 64 | 2.08--2.36 / 3.14--3.15 | 1.14--1.28 / 1.70--1.71 |
| 64 px | All live | 56 | 1.88--2.13 / 4.19--4.20 | same policy and draw |
| 96 px | All live | 35 | 1.22--1.34 / 4.21 | same policy and draw |

At 48 px, approach from near retains R0, and approach from far retains R1;
that is intentional hysteresis. The raw JSON also includes forced sprite
controls at 64 and 96 px. The normal trial policy does not choose them, and
the viewer does not present them as its default behaviour.

Refresh spikes matter. At 40 px, first-run mixed GPU means are 2.97 ms on a
refresh frame and 0.42 ms on a held frame. At 24 px they are 2.91 / 0.38 ms;
at 10 px, 3.14 / 0.31 ms. Slower refresh amortizes the bake but does not make
those spikes disappear. All eligible slots refresh together here; a bounded,
staggered refresh schedule remains unmeasured.

The switch trace holds 48 px and runs sixteen dual-render frames, eight in
each direction, three times per process. Its first frame refreshes the entry
pose; textures and pipelines already exist. Peak GPU frame time is 6.30 and
6.51 ms, above either steady representation. This is a worst-case synchronized
switch for the eligible batch, not proof of a cheap per-subject production
transition or a zoom trajectory's frame budget.

Initial whole-probe setup plus GPU completion took 241.82 ms in the first
process and 33.55 ms in the confirmation. It allocates the comparison renderers,
targets, cache and compositor, including pipeline creation. It is recorded
separately from pose-entry refresh, and neither number is a promised startup
or production cache-allocation latency.

### Storage, visual result and plan fit

The 63 used cache slots occupy a 64-tile 512 x 512 atlas: 1 MiB RGBA8 plus
1 MiB Depth32Float. Depth is shared with the bake renderer. Additional bake
workspace includes 1 MiB terrain depth, 794,848 bytes renderer buffers and
32,768 bytes sprite-instance capacity. Common display targets, the separate
comparison renderers, transition pictures, pipeline allocations and CPU source
anatomy are additional. This is logical storage, not a VRAM-profiler result.

This trial leaves cache resolution at 64 pixels. Consequently the ten-pixel
case saves only modest GPU time despite slower image refresh; the bake and
examined live body impose a floor. Lower-resolution caches and the already
planned R2 individual marks should be compared before extrapolating to very
large populations. Retaining one fixed-facing image is not qualification of
all camera, lighting, anatomy, palette, equipment or injury invalidation keys.

Plan fit: L10 owns projected-size choice, hysteresis, held-detail bounds and
R0/R1 composition. The proposed image-refresh policy belongs inside R1 and
does not pace E8 decisions or E10 native physical steps. L6 and the existing
residency policy still own production bake/parity, allocation and budgets;
the implicit offscreen render is not completion of L6's CubeCL/CPU agreement.
Game presets and examining/approach policy remain product-owned. Isocosm BD6
already reserves this far/many-token hybrid, so this trial does not change
its ordinary live-token choice. The active presentation/board plans and both
indexes were updated, without choosing product defaults or closing their
integration gates.

The captured replay uses the real PNGs; zoom selects one of six captured
sizes. Its displayed switch selects source pixels with the same ordered
pattern; timing rows come from the native GPU run, not browser timing. Native
motion plays once, slowly, and can be scrubbed. Headless Edge checks cover
48 px approached from either side, far 5 Hz mode, current image loading,
one-step playback stopping at pose 24, reduced-motion selection and the
360-pixel layout (328-pixel iframe) without root horizontal overflow. No
JavaScript errors were observed. The final fragment is below 1 MB.

Reproduce from the repository root, retaining fresh output paths:

```powershell
cargo build --release --manifest-path shared/isometer/Cargo.toml -p isometer-render --example hybrid_compare --target-dir C:/t/cargo-targets/isocosm
C:/t/cargo-targets/isocosm/release/examples/hybrid_compare.exe <fresh-output> <motion.json> [settings.json]
$env:CREATURE_COMPARE_REVERSE='1'
C:/t/cargo-targets/isocosm/release/examples/hybrid_compare.exe <fresh-confirmation> <motion.json> [settings.json]
Remove-Item Env:CREATURE_COMPARE_REVERSE
python shared/isometer/crates/isometer-render/examples/creature_compare/hybrid/report.py <fresh-output> <fresh-confirmation> <viewer.html>
```

`settings.json` may be omitted. The emitted `config` object is a reusable
settings file. Source and binary hashes are in `provenance.json`. Checks:
release build, two GPU executions and their positive/negative controls,
`rustfmt --check`, Python syntax parsing, scoped `git diff --check`,
`responsa check` and the browser checks above. No new Cargo home, worktree or
isolated target was created; the existing repository target is reused.
