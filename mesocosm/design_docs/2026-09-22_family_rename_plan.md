# The family rename: Isocosm, Eponym, Isocosm: VTT

**Date:** 2026-09-22

**Status, 2026-09-22:** in progress. R0 landed: the name reservations
`eponym`, `isocosm` and `isocosm-vtt` are published on crates.io at 0.0.1.
R1 landed the same evening: the wing's documents use the new names in the
present tense, and the drafts for the six maintainer-owned files are in
Findings for Mark's word. R3 is completed within the sim's own implementation
change: `shared/isocosm`, package `isocosm` 0.1.0, and its native bench
consumer. R2, R4 and R5 remain lanes Mark opens.

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
   is the default.
2. **The repository name** (R5). Renaming aligns the umbrella with the
   family word; not renaming keeps every existing link and pin valid.
3. **The six maintainer-owned files.** Drafts arrive under R1's Findings
   for his word.
4. **`paredros-identity` under ruling 34.** It is to be absorbed by dramatis
   under W3; whether it is renamed to `eponym-identity` first or absorbed
   directly is a sequencing choice for whoever opens R2 and W3.

## Findings

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
