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
