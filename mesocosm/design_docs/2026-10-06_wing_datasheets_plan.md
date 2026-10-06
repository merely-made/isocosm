# Wing datasheets: packs in TOML, the check, and foundings as data

**Status:** in progress, 2026-10-06. P1 and P2 are landed and verified; P3
is next.

Carries out rulings 623 to 625 of the wing design record, under the stack's
format rule in mere's `design_docs/2026-10-06_data_formats_brief.md`. F1
there: text is split by who writes it, with TOML for what people author or
review, and datasheets in Livery's shape.

## Rulings carried

- **623.** Rosters and rulesets move into datasheets, through this plan.
  Mark: "Yes, through a plan (Recommended)".
- **624.** Packs load from TOML or JSON, TOML preferred. Mark: "Accept
  both". *Reading, not ruled:* `abi` stays 1, since only the syntax changes.
- **625.** Every founding becomes data, the instrument variants too. Mark:
  "Everything, instrument variants too".

## Findings (2026-10-06, isometry `3530f611`)

- **Two pack loaders.**
  - Mesocosm's admission (`mesocosm/crates/mesocosm-phenotype/src/admit.rs`,
    `MANIFEST = "mesocosm-pack.json"`, decoding at `:315`) reads the files the
    manifest names, in that order, and refuses an undeclared `.json`.
  - The VTT's content packs (`crates/isometry-system/src/sys/generator.rs`,
    `MANIFEST_FILE = "isometry-pack.json"` at `:128`, manifest at `:141`,
    fixtures at `:214`).
- **Identity does not depend on the files.** A world records
  `Registry::digest`, "a ruleset digest, not a version string", taken over
  the admitted, canonically ordered registry. PD3's parity receipt compares
  the admitted pack with mesocosm-core's native ruleset. So the syntax of the
  pack files cannot change a world's identity, and P1 proves it rather than
  assuming it.
- **Files to move.**
  - `mesocosm/packs/mesocosm/`: the manifest, five `processes/*.json`, and
    two `fixtures/*.json`. `expression/gland.lua` stays Lua.
  - `crates/isometry-system/examples/packs/`: four `isometry-pack.json` and
    two generator fixtures.
  - The SRD data (`crates/isometry-system/data/*.json`) stays a JSON import.
- **The foundings.**
  - `Founding` (`shared/isocosm/src/legacy/mesocosm/world/genesis/founding.rs`)
    is a `Copy` enum of eight variants. Each gives a `PartPalette` and per-tier
    lists of `fn() -> Recipe` from `axis/archetype{,/branching,/jointed,/spaced}.rs`,
    about 1,200 lines.
  - A recipe is a flat list of tagmata (a count, an appendage, two shape
    selectors) plus a variance.
  - It is used at 71 non-test sites in 30 files (40 in `shared/isocosm`, 11 in
    `mesocosm-genet`), and in 49 test sites.
  - Recorded receipts store founding names as strings
    (`"palette": "SpacedRoster"`).
- **`SEEDED_KINDS`.** Six strings in `shared/wing-impresa/src/kinds.rs:16`.

## Phases

### P1: packs in TOML (rulings 623, 624)

- **Both loaders read TOML or JSON.** The syntax is chosen by the declared
  file's extension, into the same serde types, with `deny_unknown_fields`
  kept.
- **Manifests are found as `mesocosm-pack.toml` or `isometry-pack.toml`
  first**, then the JSON names.
- **The undeclared-file check covers `.toml` and `.json`.**
- **The repository's packs above are converted to TOML.**

**Done when:**
- every admission and content-pack test passes;
- a new test admits the same pack in each syntax and gets the same
  `Registry::digest`;
- PD3's parity receipt still holds;
- the converted Mesocosm pack's digest equals the JSON pack's digest at
  `3530f611`;
- the VTT's generator fixtures still produce their recorded output.

### P2: the check (F1's enforcement)

- **The pre-commit hook refuses a staged `.json` under any `packs/`
  directory** in this repository, naming the rule. It sits beside the
  600-line ceiling the hook already enforces.
- **Ruling 624 keeps JSON loadable,** so the check polices this repository's
  own packs, not packs written elsewhere.

**Done when:** a staged pack `.json` is refused (the control), and the hook
passes on the converted tree.

### P3: foundings and kinds as data (ruling 625)

- **P3a: datasheets beside the code.**
  - One TOML datasheet per founding, in Livery's shape (`schema`, owner and
    consumer, a `status` saying it is authored, `sources` naming the DC plan
    or ruling behind it), embedded in `isocosm` and parsed once.
  - Each holds the palette and the per-tier recipe lists, with shapes by
    name.
  - `SEEDED_KINDS` gets its own datasheet in `wing-impresa`.
  - Tests prove each datasheet builds values equal to the current code:
    every palette, every recipe, every kind.
- **P3b: `Founding` reads the datasheets.**
  - `Founding` stays a small `Copy` handle. Its eight names stay as they are,
    so call sites and stored receipts keep reading `SpacedRoster`.
  - The code-built palettes and recipe lists are removed once P3a's equality
    holds.

**Done when:**
- for every founding, a world founded at a fixed seed has the same state hash
  before and after, recorded in a receipt;
- every `isocosm`, `wing-impresa`, Mesocosm and Eponym test passes.

**Checkpoint before P3b:** how the datasheets are found, embedded or read
from a pack directory, decides whether foundings join the admitted ruleset's
digest. That choice goes to Mark with the evidence, if P3a shows it has more
than one defensible answer.

### Findings from P1 (2026-10-06)

- **The fixtures stay JSON, under F1.** Both of Mesocosm's
  `fixtures/gland_*_ground.json` hold `u64` values above TOML's signed 64-bit
  range: the ruleset digest `11989399369617600561`, and four recorded `draws`.
  The VTT's generator fixtures record `expected` output and
  `expected_draws`. A fixture records a run, so it is program-written, and F1
  keeps it JSON. The Findings above listed them as files to move; that was
  this plan's error, not a ruling's. P2's check exempts `fixtures/`.
- **There are three VTT manifests,** `core`, `demo` and `watchtower`, not
  four.
- **Each converted file parses back to its JSON's data:** the Mesocosm
  manifest and five process records (a script's round-trip), and the three
  VTT manifests (compared through Python's `tomllib`).
- **The new dependency changes nothing in the graph.** `toml` was already
  there: Mesocosm's ignored lock at 1.1.4, and the root lock at 1.1.6. The
  root lock gained one edge, `isometry-system` → `toml`, and no packages.

## Progress

- 2026-10-06: plan written; rulings 623 to 625 recorded.
- 2026-10-06: P1 and P2 landed on branch `wing-datasheets`. Logs are in
  `Code/testing/isometry-datasheets-*.log`.
  - **P1, Mesocosm.** `mesocosm-phenotype` gains one reader
    (`src/data.rs`) that picks the parser by extension.
    - Admission finds `mesocosm-pack.toml` first, then the JSON name
      (`MANIFEST_JSON`). Its undeclared-file walk covers both extensions and
      exempts only the manifest it read, so a second manifest is refused.
    - `Fixture::read` uses the same reader.
    - The tests pass: admission 16, the new `admission_syntax` 6, authored
      gland 18, packed gland 7.
    - PD3's parity holds over the shipped TOML pack: the admitted digest
      equals `Registry::native().digest()`.
    - The same pack admits to one digest in either syntax.
    - **Control.** With the TOML branch disabled, five of the six syntax
      tests fail. The sixth, the malformed-TOML test, holds either way,
      since a disabled reader also refuses the file as malformed.
    - One existing test changed its target file, not its intent: it now
      corrupts the shipped `secrete.toml`.
  - **P1, the VTT.** `GeneratorPack` finds `isometry-pack.toml` first, then
    the JSON name (`MANIFEST_FILE_JSON`), and refuses a pack holding both.
    Manifests and fixtures are parsed by extension. `isometry-system`'s tests
    pass, 49 and 5, including the new `a_pack_with_both_manifests_is_refused`
    and the shipped packs loading from TOML.
  - **P2.** `scripts/pack-json.ps1`, run by `.githooks/pre-commit` beside
    the line ceiling, refuses `.json` under a `packs/` path segment unless
    it is below `fixtures/`. The whole tree passes. **Control:** a staged
    `processes/stray.json` is refused with the rule's message.
