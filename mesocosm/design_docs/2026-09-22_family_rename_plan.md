# The family rename: Isocosm, Eponym, Isocosm: VTT

**Date:** 2026-09-22

**Status, 2026-09-24:** landed: R0 to R5 done (Progress). What remains is the runtime strings Mark chose to leave, and the `paredros` crate kept for a future use. R2 landed 2026-09-24: `paredros/` is
`eponym/`, the root package is `eponym` 0.0.2, the five member crates are
`eponym-*`, and the eponym workspace's test count matches its pre-rename
baseline (which was already red; see Findings). R0 landed: the name reservations
`eponym`, `isocosm` and `isocosm-vtt` are published on crates.io at 0.0.1.
R1 landed the same evening: the wing's documents use the new names in the
present tense, and the drafts for the six maintainer-owned files are in
Findings for Mark's word. R3 is completed within the sim's own implementation
change: `shared/isocosm`, package `isocosm` 0.1.0, and its native bench
consumer. R4 and R5 remain lanes Mark opens.

**Owns:** the renaming of the wing's products and the sim to the names
ruled on 2026-09-22 (wing design record, rulings 109 and 110), across
documents, crates, packages, indexes and the naming ledger. **Does not
own:** the names themselves, which are Mark's rounds, or any product's
design. **Consumes:** the [wing design record](2026-09-18_wing_design_plan.md)
and the naming ledger's family naming round of 2026-09-22.

## 1. The names

| Thing | Before | After | Crate | Status |
| --- | --- | --- | --- | --- |
| The family | the games wing | **Isocosm** | `isocosm` 0.0.1, reserved 2026-09-22 | ruled 110 |
| The sim | isotropy (ruling 17) | **Isocosm** | `isocosm` is the reservation; the implementing crate is `shared/wing-sim`, plain by the stack's tier rule | ruled 110; ruling 17 superseded |
| The second-person game | Paredros | **Eponym**, "Isocosm: Eponym" | `eponym` 0.0.1, reserved 2026-09-22; `paredros` 0.0.1 stays as history | ruled 109 |
| The tabletop | Isometry | **Isocosm: VTT**; Isometry retired as a product word | `isocosm-vtt` 0.0.1, reserved 2026-09-22; the `isometry-*` crates keep their prefix as a plain technical description | ruled 110, 111 |
| The ecological roguelike | Mesocosm | "Isocosm: Mesocosm" | `mesocosm`, unchanged | ruled 110 |
| The bridging organ | isostasy | isostasy | none yet | unchanged by this round |

Rules that hold throughout, from the denizen precedent of 2026-09-20:
historical rulings and quotations keep the old word with a supersession
pointer; `archive_docs/` is never edited; maintainer-owned files (the three
`PROJECT_DESCRIPTION.md` and the three `CLAUDE.md`) change only on Mark's
word, with drafts prepared here; and the naming ledger records each step.

## 2. Sizing, measured 2026-09-22

| Surface | Count | Note |
| --- | --- | --- |
| "paredros" in `.md`, `.toml`, `.rs`, outside `target/` | 210 files, 1,589 mentions | includes the record's verbatim rulings, which stay |
| `paredros-*` crate references | fixture 24, client 17, identity 11, social 8, world 7, sortie 5, session 3, timed 1 | package names, path dependencies, `use` lines, test names |
| Paredros workspace | `paredros/` root package plus five member crates | the root package is the reservation, at local 0.0.2 |
| "isotropy" outside the design record set | one live crate, `shared/isotropy/`, its bench binary and tests, and a path dependency from `mesocosm-genet` | a peer session's active work as of 22:36 on 2026-09-22; not touched here |
| "Isometry" as the tabletop's name | everywhere: the repository name, root `README.md`, root `CLAUDE.md`, `PROJECT_DESCRIPTION.md`, the root Cargo workspace, `scripts/wing.ps1`, seven `isometry-*` crates | the product word is retired (ruling 111); the crates keep the prefix as a plain technical description; the repository name is R5's question |

## 3. Phases and done-conditions

- **R0, the reservations.** Landed 2026-09-22: `eponym`, `isocosm` and
  `isocosm-vtt` published at 0.0.1 from minimal packages in the paredros
  reservation's shape (MPL-2.0, README, a doc comment, no code). Their
  sources are folded into the tree by R2 and R4 rather than added beside
  it. Done.
- **R1, the wing's documents.** Done when the wing design record, the sim
  plan, the session notes, the docket, the founding record, the prior-art
  brief and the three `DOC_README.md` files use the new names for the
  products and the sim wherever they speak in the present tense, keep every
  verbatim ruling as spoken, and carry a supersession line where a name
  changed; and when drafts for the six maintainer-owned files are in this
  plan's Findings for Mark's word. No archive edited.
- **R2, Paredros to Eponym.** Done when `paredros/` is `eponym/`, its root
  package is `eponym` at a version above 0.0.1 with the published
  reservation's README and doc, `paredros-*` crates are `eponym-*` with
  their paths, package names and `use` lines updated, `scripts/wing.ps1`
  selects `eponym`, the umbrella `CLAUDE.md` and root `README.md` name it,
  and `cargo check --workspace --all-targets` and `cargo test` pass in the
  eponym workspace with the same test count as before the rename. Fixture
  and receipt file names that carry "paredros" are renamed with their
  references; recorded receipts in `design_docs/` keep their historical
  names.
- **R3, the sim crate.** Completed by its owning session, alongside the first
  tested implementation, as `shared/wing-sim/`, plain by the stack's tier
  rule, with Isocosm as the name of what it implements. Done when no
  "isotropy" remains in its identifiers or its consumers.
- **R4, the tabletop.** Done when the root `README.md`, `CLAUDE.md` and
  the tabletop's product identity say Isocosm: VTT, with Isometry retired
  as a product word and the `isometry-*` crates unchanged as plain
  technical names; when `isocosm-vtt` is the
  tabletop's reservation package at a version above 0.0.1; and when the
  `isometry` crate on crates.io, reserved 2026-07-14 as "A pixel-art,
  peer-to-peer virtual tabletop", has a decision recorded (§4).
- **R5, the repository.** Done when Mark has decided whether
  `merely-made/isometry` is renamed to `merely-made/isocosm` (GitHub
  redirects the old name) and, if so, every remote, pin and link in the
  workspace that names it is updated the same session, sibling repositories
  included.

Order: R1 first, since every later phase reads it; R2 and R4 independent of
each other; R3 when the peer's crate lands; R5 last.

## 4. Decisions for Mark

1. **The `isometry` crate on crates.io.** It is the tabletop's original
   reservation and describes the tabletop. Options: keep it as a second
   reservation for the tabletop beside `isocosm-vtt`; repoint its
   description to the family; or leave it untouched as history. Leaving it
   is the default. **Taken as the default under R4, 2026-09-24:** left
   untouched at 0.0.1 as history; Mark may repoint it later.
2. **The repository name** (R5). Renaming aligns the umbrella with the
   family word; not renaming keeps every existing link and pin valid.
   **Taken 2026-09-24: renamed.** `merely-made/isometry` is
   `merely-made/isocosm`; GitHub redirects the old name.
3. **The six maintainer-owned files.** Drafts arrive under R1's Findings
   for his word.
4. **`paredros-identity` under ruling 34.** It is to be absorbed by dramatis
   under W3; whether it is renamed to `eponym-identity` first or absorbed
   directly is a sequencing choice for whoever opens R2 and W3.

## Findings

- **2026-09-24, R2: the baseline was red before the rename.** On the
  unrenamed tree, `cargo check --workspace --all-targets` passed with one
  dead-code warning (`retarget_from_ground`, now
  `eponym/crates/eponym-client/src/brick.rs:28`), but `cargo test --workspace`
  failed in `paredros-sortie`'s `tests/sortie.rs`. With `--no-fail-fast` it
  reported 230 passed, 2 failed, 3 ignored:
  `an_injury_persists_as_a_body_revision_fact` and
  `a_tag_in_occurs_mid_action_under_the_pact`. At the coordinator's word the
  done-condition became "the same counts and the same two failures". Filed
  with the likely cause (mesocosm-core's 2026-09-16 ecology commits shifting
  the grown terrain the sortie scenes rely on) in the
  [execution plan](../../eponym/design_docs/2026-08-07_paredros_execution_plan.md)
  §6 for a separate lane.
- **2026-09-24, R2: what was renamed and what was kept.** Renamed: the
  directory; the five crates' directories, package names, path dependencies,
  `use` lines and crate references; the root package (`eponym` 0.0.2, the
  published reservation's description, keywords and `lib.rs` doc, whose last
  paragraph now names the `eponym-*` crates as the implementation); the root
  workspace's `exclude`; `scripts/wing.ps1`, `scripts/audit-source-identity.ps1`
  and `eponym/build.ps1`; `paredros/` paths and `paredros-<crate>` names in
  documents outside `archive_docs/` and outside the wing design record's §0;
  and the product name in code comments, manifest descriptions, window titles
  and GPU labels. No crate named `paredros-fixture`, `paredros-session` or
  `paredros-timed` exists: those are runtime strings, and the only files whose
  names carry "paredros" are the two dated plans, which keep their names.
  **Kept, as runtime interfaces that recorded receipts depend on,** and left
  for Mark: the `PAREDROS_*` environment variables, the `paredros-fixture:`
  glyph namespace (in the 2026-09-14 acceptance receipts, asserted by
  `eponym/testing/session/acceptance.scenario` and isomere's journal test),
  the `Code/testing/paredros` capture directory, receipt fields
  `vessel: "paredros"`, the `paredros.session/` event URIs, the
  `paredros-session` probe kind and temp-directory names. The eponym
  `Cargo.lock` is gitignored, so no lock change is committed.
- **2026-09-24, R2: departures and open names.** (a) The `paredros-room`
  precedent kept old package names in dated verification entries; R2
  followed its brief instead and rewrote `paredros-<crate>` in dated commands
  too, so they run today. Lines whose subject is a dated rename (the
  2026-09-13 client rename, the 2026-09-09 import under `paredros/`) keep the
  old name with a pointer to R2. (b) §4 decision 4 was taken as "rename
  first": `paredros-identity` is `eponym-identity`, and absorption into
  dramatis under W3 is unchanged. (c) The wing design record's ruling that
  `paredros-world` becomes `paredros-core` (§9.1, and the evaluations' §3.1)
  is kept as spoken; whether the future core is `eponym-core` is Mark's
  word. (d) Draft 5 was applied without its sentence about the directory
  keeping its name until R2, and draft 6 changed only the lead sentence, so
  the prose "Paredros" remains in the rest of `eponym/CLAUDE.md` and
  `eponym/design_docs/PROJECT_DESCRIPTION.md` (three in each besides the
  drafts' "formerly", the latter's title included), both maintainer-owned. Across current Markdown outside this plan, 438
  capitalised "Paredros" remain in 53 files, mostly Eponym's own dated plans; R2 changed
  paths and crate names there, not prose.
- **2026-09-24, R2 residue:** the done-condition's grep (`paredros`,
  case-sensitive, in `.md`, `.toml`, `.rs` and `.ps1` outside `target` and
  `archive_docs/`) returns 221 lines: 35 in this plan; 8 in the wing design
  record's §0; 15 citing the two dated plan file names; 34 naming the
  historical `paredros-room`; 86 runtime interfaces kept as above (25 glyph
  namespace, 32 capture paths, 29 other strings, labels and temp names); 3
  on the old reservation and repository locations; and 40 dated history
  lines (gate lists reading "paredros check", cargo-home and log names, the
  rename pointers, the word's etymology, the ruled `paredros-core`).

- **2026-09-22, R3:** the owning sim session incorporated ruling 110 before
  landing its work. The temporary `shared/isotropy` and `shared/wing-sim`
  paths became `shared/isocosm`, package `isocosm` 0.1.0. Its binary is
  `isocosm-bench`; tests and `mesocosm-genet` consume `isocosm`. The crate
  compiles independently and the native bench scenarios exercise its real
  consumer. This is local implementation, not a new crates.io publication.
  The earlier measurements below retain their historical paths.

- **2026-09-22, R1 landed.** Nine documents renamed in the present tense by
  one rule: "Paredros" to "Eponym" and "isotropy" to "Isocosm" outside
  double quotes, outside code spans and outside the record's §0, with a
  names line under each document's date. Counts before and after: the wing
  design record 71 to 15 (the remainder are §0's verbatim rulings and
  quotations), the founding record 43 to 2, Mesocosm's index 18 to 1,
  Paredros's index 13 to 1, the rest under ten. Lowercase paths such as
  `paredros/crates/...` and `shared/isotropy` are untouched because they are
  true until R2 and R3.
- **2026-09-22, drafts for the six maintainer-owned files,** each a
  replacement for one passage, for Mark's word:
1. *Root `CLAUDE.md`, Project Identity, first sentence:* "**Isocosm: VTT**
  (formerly Isometry; renamed 2026-09-22, wing design record rulings 110
  and 111) is a pixel-art isometric virtual tabletop over the Isocosm
  simulator: a P2P map editor and turn-based play substrate for D&D,
  Pathfinder, and other systems. Its crates keep the `isometry-` prefix as
  a plain technical name." And under the umbrella heading, after "This Git
  repository contains three products": "The family is Isocosm; the
  products are Isocosm: VTT, Isocosm: Mesocosm and Isocosm: Eponym, over
  one simulator."
2. *Root `design_docs/PROJECT_DESCRIPTION.md`, the lead:* the same first
  sentence as above, with the existing pillars unchanged.
3. *`mesocosm/CLAUDE.md`, the vessel line:* "Vessel 1 of the Isocosm family,
  Mesocosm (first person), Eponym (second person) and the VTT (third
  person), over one simulator, Isocosm, sharing a world substrate, a
  lineage model, and a trust plane."
4. *`mesocosm/design_docs/PROJECT_DESCRIPTION.md`:* "Paredros" to "Eponym"
  wherever it appears, and the family named as in draft 3.
5. *`paredros/CLAUDE.md`, the title and identity:* "CLAUDE.md, Eponym
  Repository Role"; "**Eponym** (formerly Paredros; renamed 2026-09-22, wing
  design record ruling 109) is a second-person action RPG in a persistent
  generated world." Plus one sentence under the location note: "The
  directory and crates keep the `paredros` name until the rename lane's R2
  lands." The vessel line as in draft 3, with Eponym as vessel 2.
6. *`paredros/design_docs/PROJECT_DESCRIPTION.md`, the lead:* "Eponym
  (formerly Paredros) is a second-person action RPG in a persistent
  generated world."

- **2026-09-22:** the paredros reservation is the `paredros/` workspace
  root package (`paredros/Cargo.toml`, local 0.0.2, published 0.0.1), with
  a doc-only `src/lib.rs` and a README; the new reservations copy that
  shape.
- **2026-09-22:** a peer session's `shared/isotropy/` crate appeared during
  the naming round, already consumed by `mesocosm-genet`'s bench
  (`mesocosm-genet/Cargo.toml:27`), with a bench binary and tests; R3 waits
  on it.
- **2026-09-22, priority:** this repository was bootstrapped 2026-07-05 and
  the `isometry` crate reserved 2026-07-14; Dark Mode Games' Isometry
  announced its beta 2026-07-06 after about three months of work; neither
  side holds a mark. Subtitling the tabletop under Isocosm is the response.

## Progress

- 2026-09-22: R3 folded into the owning sim session's first implementation;
  core, binary, tests, native consumer and current indexes use Isocosm.

- 2026-09-22: plan written at Mark's word ("Let's plan the rename too"); R0
  landed the same day.
- 2026-09-22: R1 landed at Mark's word ("Sure"): nine documents renamed in
  the present tense, drafts for the six maintainer-owned files filed.
- 2026-09-22: ruling 111 (option B) applied: Isometry retired as a product
  word; the names line, the names table, R4 and two drafts amended; R3's
  wording brought to the owning session's `wing-sim`.
- 2026-09-24: ruling 112: Mesocosm keeps its name and Eponym stays, so the
  names table is final; hagiograph, redshank and ortet claimed on crates.io
  at Mark's word, beside the family's three.
- 2026-09-24: R2 landed at the coordinator's word: `paredros/` to `eponym/`,
  five crates to `eponym-*`, root package `eponym` 0.0.2. Baseline and final
  alike: `cargo test --workspace --no-fail-fast` 230 passed, 2 failed (the two
  pre-existing sortie receipts), 3 ignored, over 48 test binaries;
  `cargo check --workspace --all-targets` clean but for the one pre-existing
  warning; root `--all-features --all-targets` and Mesocosm checks pass.
- 2026-09-24: R4 landed: drafts 1 to 4 applied to the root `CLAUDE.md` and
  `PROJECT_DESCRIPTION.md` and to Mesocosm's, at Mark's word;
  `crates/isocosm-vtt` added to the tabletop workspace as its reservation
  package at 0.0.2 with the Isometry clause removed, and published; the
  `isometry` crate left as history (decision 1, default).
- 2026-09-24: R5 landed at Mark's word: the GitHub repository renamed to
  `merely-made/isocosm` with its description set to the family's; the local
  remote repointed; every `repository` field in the tree's manifests and
  READMEs rewritten; the root README rewritten to present the family with
  the VTT as one section; the org profile README and the mer3ly.net site
  content updated in their own repositories the same day. The checkout
  directory stays `repos/isometry`, since local paths are not the
  repository's name. Eponym's leftover prose finished and `eponym-core`
  ruled, beside R5.
- 2026-09-24: the site deployed green at merelyllc.com c1b8ab1 after two
  smoke fixes (the headed smoke still named mesocosm as its text-first
  profile and carried the old projection counts; retinue and the measured
  counts took their place). Live: `mer3ly.net/projects/isocosm/` serves
  the Isocosm profile; `/projects/mesocosm/` is 404, as intended.
