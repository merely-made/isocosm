# Native specimen bench checks

Run from the Mesocosm workspace. Use Cargo to select the current executable;
this machine's configured target directory differs from `mesocosm/target`.

```powershell
cargo run --release -p mesocosm-genet -- --bench --seed 1
cargo run --release -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/acceptance.scenario --receipt <output>/acceptance.json --capture <output>/acceptance.png
cargo run --release -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/habitat.scenario --receipt <output>/habitat.json --capture <output>/habitat.png
cargo run --release -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/failure.scenario --receipt <output>/failure.json --capture <output>/failure.png
```

Replace `<output>` with a local directory. The first two scenarios must exit
zero. The deliberately false assertion in `failure.scenario` must exit one,
write `ok: false`, and retain fresh captures. Named captures go beside the
final PNG; the final PNG uses the exact supplied path. A native window close
defers exit until its requested final capture and receipt finish.

The scenario uses actual host pointer routing. Its pixel checks compare the
viewport content independently of the controls, exclude the overlaid button,
and use explicit fixture geometry for the authored transform. Full-resolution
PNGs remain in the local capture directory; compact JSON is retained under
`receipts/2026-09-13/`.

Source ownership, qualified acceptance and the remaining trial/cost work live
in the [wing plan](../../design_docs/2026-09-11_orthographic_voxel_presentation_plan.md#bench-b-one-interactive-genet-viewport).

## Proportion comparison

Choose **Compare proportions** to retain the current specimen beside four
single-stretch shape alternatives. **More proportions** advances through the
finite set of available edits. **Shared scale / fit each** changes only the
comparison cards; the large selected view is always framed for inspection.
Use **Save comparison** to save a new JSON file beside the configured capture
output. Its full path appears in the notice. Reopen it with:

```powershell
cargo run --release -p mesocosm-genet -- --bench --comparison <saved-file.json>
```

The native scenarios are `proportions.scenario` (use seed 1) and
`proportions-reopen.scenario` (use the resulting saved comparison). The first
checks the retained original, selection, camera-scale setting, generation of
more options and export. The second checks restoration. A changed expected
hash or mismatching content palette must fail before the native host starts.


## Generation controls

Open **Generation controls** for axial, branched or generated body plans and
raccoon-like, cat-like, horse-like, bird-like, fish-like, grass, shrub and tree
starting anatomy. **Reroll** advances body variation without changing habitat;
**Random seed** chooses and displays a new reproducible seed. Seeds 1, 7 and 42
are checked starting points. Enter a seed or mass and choose **Apply seed and
mass**. Invalid input refuses without replacing the current specimen.

Sizes 1-3 change the admitted content and authoritative body geometry. Sensory
and role-sensitive detail retain their original size. Mass is separately paid
tissue and reserve; the panel reports actual body mass and capacity. A large
body does not receive free matter. **Save specimen** preserves the seed,
criteria, content, base size and world hash. **Save criteria** exports only the
request and therefore does not preserve resized content.

`generation.scenario` exercises all eight starts, generated reroll/replay,
size, native mass editing and export. Reopen a saved specimen with the same
`--bench --comparison FILE` command as a saved proportion comparison.

`generation-reopen.scenario` checks preserved size and return to the base;
`generation-refusal.scenario` checks an unfundable request and save refusal.
Compact native receipts and source identities are in
`receipts/2026-09-13/generation/`. The starting anatomies are structural
prototypes; finished species likeness and surface markings remain open.

## Compose anatomy

**Compose anatomy** separates layout from organ pattern. Choose chain, radial,
crown, mat, vine or roots; combine with bare sites, legs, wings, fins, leaves or
feelers. Structural stretch count excludes feeding supports. Length is the seeded
maximum segments per stretch. Feeding organs remain present at bare sites;
leaves selects Producer. The seed and variation determine the remaining draws.
Selecting a starting anatomy or ordinary body plan leaves composition mode.

These controls use the ordinary recipe, content and matter pipeline. Radial
branches follow cardinal directions; roots branch down and vines bend. They
do not add flight, swimming, climbing or root physiology to the world.

`structure.scenario` exercises combinations, counts, replay and saving.
The same settings can be supplied to `generate-start`, for example:

```powershell
cargo run --release -p mesocosm-genet --bin generate-start -- --seed 1 --layout radial --organs feelers --stretches 4 --length 3 --role consumer --output generated-radial
```

Use a fresh output directory. Request JSON and **Save specimen** retain the
composition settings. Old requests without those settings keep their existing
generation streams.

`structure-reopen.scenario` uses the composition scenario's saved specimen.
`structure-cli.scenario` checks a CLI radial/feelers request at seed 1, six
stretches, length 3, consumer, against the independently observed bench hash.
Both CLI generation and native loading select the rich bench palette for
explicit structures; earlier CLI requests preserve their original palette.
Compact receipts live in `receipts/2026-09-13/structure/`.

## Glyph effect experiment

Choose **Effects experiment** for a separate 2D plane. Quotes, slashes and
backticks combine with stream, enclosure and inscription behavior. Stone,
metal and moss are receiver samples. The guaranteed demonstration profile
reflects from stone, splits at metal and binds to moss. Generated rules select
stable responses from the interaction seed. Appearance reseeding changes
the marks without changing the receiver rule.

The report explains whether the combination admits pairs, a string or separate
marks. Play/pause uses a fixed experiment clock; Step advances ten ticks and
Reset returns to zero. The specimen world does not advance. Contact timing
is scripted for this plane, not world collision or body attachment.

**Save experiment** writes a fresh JSON file beside the capture path.
**Reopen experiment** restores that file, including its tick. After restarting:

```powershell
cargo run --release -p mesocosm-genet -- --effect-experiment <saved-file.json>
```

`effects.scenario` checks native controls, changed animation pixels, unchanged
guaranteed-rule pixels after interaction reseeding, independent appearance,
split/string outcomes, restoration and specimen hash preservation.
`effects-reopen.scenario` checks restart replay, playback and pause using the
first scenario's save. Unsupported versions or invalid counts must refuse
before host creation. D&D/PF2e rule packs and scene-attached effects remain
future consumers of the documented distinction.

## Spatial glyph preview

Choose **Spatial glyphs** in the specimen view. Eighteen opaque glyphs orbit
the posed body's bounds in 3D, with camera-facing faces and shared scene depth.
**Step orbit** advances fifteen ticks; **Reset orbit** restores zero. Play/pause
uses a 33 ms presentation tick. **Change glyph** cycles quotes, slashes and
backticks; **View angle** switches between oblique and across cameras.
This preview does not mutate the specimen or implement world interactions.

Run `spatial.scenario` with the ordinary bench command above. It selects the
cat-like starting anatomy, checks orbit changes and exact reset pixels, two
angles, glyph changes, and unchanged world hash. The independent GPU test
`isometer::glyphs::tests` checks hidden rear strokes and visible edges against a
real voxel cube from two cameras. Solid strokes are the admitted first slice;
transparent overlap, surface attachment and glyph picking remain future work.

### Opaque spatial coverage

The form buttons select Orbit, Surface, Tether and Emission. Surface and
Emission can use a selected part. Tether requires two parts, so clear selection
to use the body's anchors. Faces are actual posed mesh rectangles, including
continuous body turning. Reseed spatial changes appearance; Glyph count cycles
6/18/64/128. Save/Reopen spatial replays settings and tick on the same specimen,
pose, camera and selection; it does not reload a world or provide a CLI import.

`spatial-coverage.scenario` exercises all 24 form/glyph/camera combinations,
seed/count/replay, maximum density, selected attachments and unchanged world.
Focused tests are `isometer::glyphs::tests`, `isometer::anchors::tests`
and `app::bench::spatial::sampling::tests`. This matrix covers opaque rendering
and attachment. Transparent compositing and causal world interactions remain
separate acceptance work.

## Disposable world trial

Choose **World trial** to run the currently displayed fresh specimen in its
habitat. **Step world** applies one ordinary Idle tick. **Play world** advances
at ten ticks per second; pause, reset and exit remain explicit controls. The
runtime stops at a checkpoint or 128 applied ticks. Reset restores the exact
baseline, including pending founding records. Exiting returns to unchanged
generation; rerolling or changing generation cancels the trial.

Slashes indicate recorded movement and quotes indicate recorded feeding at
organism locations. They do not identify physical feet, mouths or contacts.
The cached marks live for eight simulation ticks and are capped at 128.
**Mark height** cycles 0/2/4/6/8 world units (default 4); **Mark size** cycles
0.7/1.4/2.8 (default 1.4). These raised activity indicators keep their recorded
coordinates unchanged. Changing their presentation does not step the world.
Repeated redraws do not emit events or advance time. Hide and the planar
experiment pause playback. Save specimen continues saving source generation.

`world-trial.scenario` saves a seed-7 consumer specimen, steps 48 times (or to
an earlier checkpoint), compares paused/redrawn activity, resets and replays,
and checks source identity on exit. `world-trial-play.scenario` covers playback,
pause, hiding, switching experiment and reroll cancellation. A generated
`world-trial-reopen.scenario` receipt reopens the saved specimen with
`--comparison FILE` and applies the same Idle sequence.

Focused runtime tests: `runtime::trial::tests`. Freshness admission deliberately
refuses advanced snapshots, whose history and checkpoint must accompany them.
The prior trial report's predation attribution is corrected from donor to eater;
its regression test is `world::generation::trial::tests::predation_is_credited_to_eater_not_victim`.

Native receipts and source identities: [world-trial receipt](receipts/2026-09-13/world-trial/source.json).

## Population cost workload

The separate `--population FILE` bench input admits a presentation workload
before GPU creation. It keeps ecology and saved specimens out of benchmark
population construction. Body count and design count are independent; the
three classes are shared geometry with instance colour, rearranged reusable
parts, and seeded exterior occupancy changes. The latter is a bounded shape
fixture, not a claim of general anatomy generation.

The normal Genet document and texture-producer boundary remains in use.
Natural/Warm/Cool exercise resolved CSS appearance; turn/reset exercises
instance pose. Configurations use the same fixed orthographic camera and
world-to-pixel scale at a given viewport size. The workload reports actual
unique meshes, assemblies, instances and clipping separately.

Scenarios can bracket CPU cost phases with `cost-begin NAME` and `cost-end`.
Collection precedes scenario dispatch, so a phase begins on the following
frame. Each phase reports its first eligible profile and median/p95/max for
all eligible profiles and for the remainder after the first. Capture frames
are retained but excluded from distributions. Producer calls and actual scene
redraws are distinct counts. These are CPU wall times from the pinned Cambium
host, not GPU durations or a guarantee that each callback presented.

Example input: `{"kind":"geometry","bodies":1000,"designs":128,"seed":7,"mesh_capacity":1024}`.
Kinds are `appearance`, `assembly`, and `geometry`. Run:

```powershell
cargo run --release -p mesocosm-genet -- --population FILE --frames 1800 --scenario testing/bench/population.scenario --receipt OUTPUT.json --capture OUTPUT.png
```

[The retained matrix](receipts/2026-09-13/population/matrix.json) includes actual
counts, foreground coverage, first/steady CPU stage distributions and build-load
snapshots. Raw frame profiles and PNGs remain in the recorded local artifact
directory. The workload uses the body renderer directly, so its figures exclude
ecological projection, terrain and effects. Fixed native scale is deliberately
sparse; the independent voxel/depth tests use magnified framing.

### Dense coverage and depth overlap

Population inputs also accept `camera_extent` (8..2000 world-unit half-height),
`grid_spacing` (1..128), `depth_layers` (1..32), and
`reverse_instances` (boolean). Defaults preserve extent 500, spacing 24 and
one layer. The v2 workload digest records these settings. Layers occupy
distinct positions 40 world units apart along the camera axis; equal
projections do not mean coincident surfaces. Tight lateral spacing is a
rendering stress fixture and may interpenetrate; it is not an admitted ecology.

Use the same existing population scenario. Compare foreground coverage
against the known clear colour, not the image's most common colour, which
can become a body colour in dense scenes. Receipts separate submitted bodies,
conservative bounds crossing a clip plane, and bounds wholly outside a plane.
Input reversal does not reverse volume-key grouping inside the renderer.

A separate ignored test, `population_completion_receipt`, accepts explicit
`MESOCOSM_POPULATION_COMPLETION_INPUT` and
`MESOCOSM_POPULATION_COMPLETION_OUTPUT` paths. Its manifest contains width,
height, samples and named config cases. It uses a private baseline-feature
device and reports its adapter. Queue draining precedes each timed sample;
render/submit, bounded completion wait and serialized total are separate.
Cold setup, preparation and three warmups are outside steady distributions.
This includes CPU and GPU completion; it is not GPU-only timing or native
document/presentation cost.

[Dense/layered receipts](receipts/2026-09-13/density/source.json) retain the 23
inputs and both measurement paths. Native foreground reaches 93-95% in crop
cases; explicit clipping counts accompany those figures. A four/eight-layer
oracle verifies unchanged silhouette and doubled body intersections per ray.
Runtime-generated anatomy, terrain/effects together and transparent blending
are outside this fixture receipt.
