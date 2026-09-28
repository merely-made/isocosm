# Native specimen bench checks

## Isocosm simulation

Choose **Simulation** to found and run a world through `shared/isocosm`.
**Reservoir world** and **Ecology world** select two generator families;
seed, population, sites, lineages and ticks per step are editable. Inspection,
remembering, reproduction and record reckoning are logged sim commands.
**Compare individual replay** verifies the saved history through the other
execution mode. **Branch world**, **Preview merge** and **Apply reviewed
merge** retain both played worlds until a proposal is accepted.

```powershell
cargo run -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/sim.scenario --receipt <output>/sim.json --capture <output>/sim.png
cargo run -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/sim-ecology.scenario --receipt <output>/ecology.json --capture <output>/ecology.png
```

The first scenario checks inspection, paid birth, save/restore, branching and
merge acceptance. The second edits founding parameters, runs ecology for an
epoch, checks matter conservation and cross-mode replay, then verifies an
invalid founding leaves the played world intact. Both use native controls.
The core's [README](../../../shared/isocosm/README.md) names the implemented
behavior and the remaining sim-plan work.

### Resource-complete panel text capture

`sim-paint.scenario` opens the actual simulation panel, inspects an individual,
and stays paused at tick 0. Set `MESQUITE_CAPTURE_PAINT=1` to save a full
postcard `PaintEnvelope` beside each scenario PNG. The JSON capture record's
`paint_path` identifies the matching packet. Its fonts retain their bytes and
collection indices; glyphs retain their caller-shaped IDs and positions.
The paint packet and PNG come from the same presented frame. Existing paint
packets are refused, so choose an unused output stem for a fresh run.

From this `mesocosm` workspace (replace `OUTPUT` with a capture directory):

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cargo-targets\isometry'
$env:MESQUITE_CAPTURE_PAINT = '1'
cargo run --locked -p mesocosm-genet -- --bench --seed 1 --frames 1800 --scenario testing/bench/sim-paint.scenario --receipt OUTPUT/sim.json --capture OUTPUT/sim.png
Remove-Item Env:MESQUITE_CAPTURE_PAINT
cargo run --locked -p mesocosm-genet --example paint_replay -- OUTPUT/sim-sim-founded.paintlist OUTPUT/sim-sim-founded.png OUTPUT/replay.png
```

`paint_replay` checks lossless decoding, valid embedded faces, matching font
bytes/indices and positioned glyphs after translation, and rejection of a
deliberately removed font palette. It rasterizes the packet with Classic at
the paired PNG's size, reports pixel differences, and fails differences above
one channel value. It performs no layout or shaping. External GPU texture
references refuse replay: capturing the live Isometer producer remains a
separate integration task. Export overhead is excluded from timing claims.

The [2026-09-27 receipt](receipts/2026-09-27/paint-capture/source.json)
records a native 2464 by 1504 capture and an independent Classic replay with
zero changed pixels: 221 commands, 73 text runs, 1,228 glyphs and three full
font resources. Missing-font and existing-packet controls both reject. The
simulation remains at tick 0 with 262 entities and the same world hash.
Raw packets, paired PNGs, lock snapshots and logs stay in the receipt's local
artifact directory; embedded system-font bytes are not checked into Git.
The replay output path must also be unused.

The host integration pins Mere `ac41628a`, Genet `92b249af` and NetRender
`9607d16f` together across the shared Isomere/Isometer consumers. Default
Isometry and Eponym client checks cover those sibling graphs. The optional
Cleromancy integration retains its separately pinned graph and is outside
this capture receipt. This proves the paused panel's paint replay, not Hybrid
text rendering, live Isometer texture import or whole-loop performance.

The core's own `isocosm-scale` binary measures what it runs by size, outside
this host: time per tick, evaluations, stored groups, history growth and heap,
for both generator families and both execution modes, with fitted scaling
curves. Its receipts are in `receipts/2026-09-25/isocosm/`: `scale.json`, its
larger rungs in `scale-extension.json`, a living history in
`scale-living.json`, and `analyze.py`, which rederives `analysis.json` from
those three. `viability.json` and `ablation-matter.json` are diagnostics: which
founding keeps the ecology family reproducing, and what one per-evaluation
cost amounts to. `source.json` names the commit behind each.
`remeasure-ecology.json` runs those receipts' ecology points again after the
core stopped cloning the world and summing its matter for every act: the same
draws, every hash and count unchanged, and each point's speedup.
`differential.json`, with its driver `differential-drive.rs`, compares every
receipt and state hash of random acts on the core as it was before that
change and the two preceding it, and as it is after them.
`remeasure-source.json` names their commits.

`isocosm-probe` runs ruling 113's check on ruling 115's competition for food,
the exact individual runner against a crowd. Its certified receipt is
`probe.json` in the same directory, with density-ladder receipts beside it and
their own `probe-source.json`. `verify.py` recomputes every distance and
equivalence verdict from the raw per-draw readings, independently of the Rust
check, and re-runs the difference tests with its own permutations.

`receipts/2026-09-26/isocosm/` holds checkpoint 2 of the batch ruled 217 to
240. `probe.json` is the certified check again, under the fight in rounds of
rulings 221 to 223 and the mind of ruling 227. `probe-water.json` contests
water as well as food, two competitions a tick settled at its end (rulings 236
and 240). `probe-approximate.json` adds the crowd with ruling 220's
approximate pairing draw, at 512 to 1,024 members per site and lineage, and
the `probe-crowds-*.json` receipts time the two crowds alone at higher
densities. Each `*-verify.txt` is `verify.py`'s output on its receipt.
Under ruling 255 the three large raw receipts, `probe.json`,
`probe-water.json` and `probe-approximate.json`, live out of tree; their
path and hashes are in that directory's `RAW_RECEIPTS.md`.
`remeasure-237.json` runs the step-1 ecology points again after ruling 237's
two cuts, and `remeasure-237-comparison.json`, written by `remeasure_237.py`,
sets each point beside checkpoint 1's time. `differential.json`, with its driver
`differential-drive.rs`, compares the core before checkpoint 1 with the final
one. `checkpoint-2-source.json` names the commit behind each run.

The same directory holds the scheduler index of ruling 258, which has a due
process visit only the groups carrying the traits it requires.
`scheduler-lineages.json` sets the lineage sweep, re-measured on the core
before and after, point beside point; `scheduler_lineages.py` rebuilds it from
the raw re-measures, which live out of tree with their hashes in
`RAW_RECEIPTS.md`. `scheduler-differential.json` compares the driver's logs
before and after with the evaluation counts ruling 259 changes set apart
(`compare_logs.py`), and explains the budgeted sessions' new baseline
(`tight_baseline.py`). `scheduler-source.json` names the commits, commands
and checks.

Checkpoint 4, rulings 284 to 287, is in the same directory under `c4-`: an
advance limited by its work rather than its ticks, the budget counting the
members an evaluation stands for, groups filed as ready for a process by its
own thresholds, and feeding by a draw weighted by what each prey holds. The
certified check ran again with a lineage hunting the others by that draw,
with food and with water; `c4-probe-predators-verify.txt` and
`c4-probe-predators-water-verify.txt` are `verify.py`'s output on the two raw
receipts, which live out of tree with the rest of the checkpoint's raw runs,
their hashes in `RAW_RECEIPTS.md`. `c4-density.json` is the density ladder
with hunters and without, written by `c4_density.py`; `c4-lineages.json` sets
the lineage sweep after ruling 286 beside it after ruling 287
(`lineages_287.py`); `c4-differential.json` compares the driver's logs after
each (`differential-drive-287.rs`); and `c4-boundary.txt` records how often
prey ran out part way through a site's hunters, measured with an instrumented
copy of the core (`boundary_setup.py`, `boundary-measure.rs`).
`checkpoint-4-source.json` names the commits, commands and checks.

Checkpoint 4b is beside it under `c4b-`: a reading of what each hunt's prey
held as its meals began, and a wider hunting domain in which prey carry fat
that only a hunt takes. `c4b-probe-predators-verify.txt` and
`c4b-probe-predators-water-verify.txt` are `verify.py`'s output on the two raw
certified receipts, which live out of tree, hashed in `RAW_RECEIPTS.md`.
`c4b-boundary.txt` records how often prey ran out part way through a site's
hunters in the domains tried, measured with an instrumented copy of the core
(`boundary_setup2.py`, `boundary-measure-4b.rs`), and why the certified one
was chosen. `checkpoint-4b-source.json` names the commit, commands and checks.
A crowd refusing a draw has it recorded and left out of that arm's
comparisons; the crowd under certification fails the check if it refuses more
than one draw in a hundred, and the controls' refusals carry no bound.
`verify.py` applies the same rule and, where the worlds hunted, prints each
arm's refusals beside its bound.

`receipts/2026-09-27/isocosm/` holds checkpoint 5, Part B's steps 1 to 3:
part shapes and the function catalogue, with the part a process binds
(rulings 338 to 341 and 360); declared conversions, the diffusion kernel and
the dev source (342 to 344, 357 and 358); and the flow record (345 and 359).
`c5-draws.json`, rebuilt by `c5_summary.py` from the two raw runs of
`c5-draws.rs`, which live out of tree with their hashes in `RAW_RECEIPTS.md`,
summarizes 1,000 clock-seeded draws of each family, each run individually and
grouped: over the whole function catalogue, where the two modes agreed in all
1,000 and every one of the 510 functions was bound; and over ecology worlds
whose transforms declare their conversions, with dev placements and host acts
between advances, where every member's and site's ledgers reconciled with the
flow record at every step of all 1,000, matter was always the founding total
plus what was issued, and the modes claimed the same moves.
`c5-differential.json`, with its driver `differential-drive-340.rs`, finds
the driver's logs on main before Part B and at this checkpoint identical line
for line, and the probe's pilots with food, water and hunters, compared by
the previous day's `same_but_time.py`, equal in every field but wall time. No
crowd arm applies: the crowd refuses processes binding a part until the
vertical probe certifies them, and the flow record and the dev source are the
core's alone. `checkpoint-5-source.json` names the commits, commands, checks
and the two source tests not ported.

**2026-09-27, ruling 371 follow-up:** the flow record now returns owned
moves for one explicit tick or one host command; the until-drained queue is
removed. `c5-per-tick-draws.rs` and `c5_per_tick_summary.py` produce
`c5-per-tick.json`: 1,000 worlds, 95,824 handoffs in each mode, every ledger
reconciled, every mode comparison agreed, and 158,312 earlier-tick injection
controls detected. All historical final states and issued matter match.
Explicit one-tick collection changes cohort batching, so flow-row counts are
not claimed byte-identical to the old multi-tick run. The four unrecorded
regression logs still match all 128,676 historical lines. Two deliberate
core faults, retaining old moves and dropping a move, make the flow tests
fail. `checkpoint-5-per-tick-source.json` records the source and qualified
checks; `PER_TICK_RAW_RECEIPTS.md` hashes the complete fresh raw/log files.
Reserve physiology, filial development cost and checkpoint 6 remain outside
this receipt. The historical checkpoint-5 artifacts above retain their words.

## Specimen checks

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

`expression-sever.scenario` shows the two states a glyph mark can be drawn in.
A bound seed-7 consumer embodies the base glyph from tick zero, so its marks
are slashes inscribed on the faces of the parts expressing it. One carve is the
whole journey: it earns the glyph once, and this body never moves or feeds, so
the grant count stays at one for the rest of the run. Forty steps later the body
is gone and no living part expresses anything, so the same effect is drawn
through the journey instead — amber quotes rising camera-facing at the recorded
place, with no face to anchor to. The journey reading and the grant count are
byte-identical either side of the change. The bench has no control that severs
a part, so the removal is the world's own death of the bound body; both paths
leave the embodied reading the same way, through living parts. Framing follows
uptake-world (UI zoom 0.75, mark size 2.8) because the marks are a few pixels
each at the default.

Focused runtime tests: `runtime::trial::tests`. Freshness admission deliberately
refuses advanced snapshots, whose history and checkpoint must accompany them.
The prior trial report's predation attribution is corrected from donor to eater;
its regression test is `world::generation::trial::tests::predation_is_credited_to_eater_not_victim`.

Native receipts and source identities: [world-trial receipt](receipts/2026-09-13/world-trial/source.json),
[expression receipt](receipts/2026-09-15/expression/source.json).

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
