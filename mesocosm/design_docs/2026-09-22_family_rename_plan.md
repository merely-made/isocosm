# The family rename: Isocosm, Eponym, Isocosm: VTT

**Date:** 2026-09-22

**Status, 2026-09-22:** plan. R0 landed the same day: the name reservations
`eponym`, `isocosm` and `isocosm-vtt` are published on crates.io at 0.0.1.
Nothing in the tree is renamed yet; every later phase is a lane Mark opens.

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
| The sim | isotropy (ruling 17) | **Isocosm** | `isocosm` | ruled 110; ruling 17 superseded |
| The second-person game | Paredros | **Eponym**, "Isocosm: Eponym" | `eponym` 0.0.1, reserved 2026-09-22; `paredros` 0.0.1 stays as history | ruled 109 |
| The tabletop | Isometry | **Isocosm: VTT**, also "Isocosm: Isometry" | `isocosm-vtt` 0.0.1, reserved 2026-09-22; the `isometry-*` crates keep their names | ruled 110 |
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
| "Isometry" as the tabletop's name | everywhere: the repository name, root `README.md`, root `CLAUDE.md`, `PROJECT_DESCRIPTION.md`, the root Cargo workspace, `scripts/wing.ps1`, seven `isometry-*` crates | the word stays as the tabletop's subtitle and the crates' prefix; what changes is the product's lead name and the family's |

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
- **R3, the sim crate.** Done when the crate the peer session is building
  as `shared/isotropy/` is `shared/isocosm/`, package `isocosm` at a version
  above the reservation, its binary and tests renamed, and its consumers
  repointed; coordinated with that session rather than done over it, and
  not before it lands.
- **R4, the tabletop.** Done when the root `README.md`, `CLAUDE.md` and
  the tabletop's product identity say Isocosm: VTT with Isometry as the
  subtitle and the `isometry-*` crates unchanged; when `isocosm-vtt` is the
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

- 2026-09-22: plan written at Mark's word ("Let's plan the rename too"); R0
  landed the same day.
