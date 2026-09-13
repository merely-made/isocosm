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
