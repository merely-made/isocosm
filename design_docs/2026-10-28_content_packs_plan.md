# Content packs

**Status:** 2026-10-28: cut from `2026-10-17_private_pack_authoring_plan.md`, `2026-08-08_extracted_receipts.md` and the pack parts of `2026-10-15_campaign_retirement_plan.md`, which are archived in `archive_docs/2026-10-28/`, with the Massif Press ask from the sim plan (`mesocosm/design_docs/archive_docs/2026-10-28/2026-09-22_sim_plan.md`). The private terminal authoring tool `pack-author` is landed (`crates/isometry-system/src/pack_author/`, feature `pack-authoring`) over `ContentPackManifest` as the schema and `GeneratorPack::load` as the loader. The manifest carries optional `license` and `credit` (blank refused by `validate`). Nothing outside tests reads those terms yet. Open: showing terms to the table, authoring them, the appearance preview and a real tileset, distribution, and the licences of rulesets that may ship.

**For:** packs as the unit content ships in: what a pack declares about its terms and how the VTT and the authoring tool treat them, the authoring tool's residue, how packs are distributed, how a pack's appearance is previewed and given a real tileset, and which rulesets may ship in a pack. Mark's doctrine statements are Licensing 1 (rulesets), Licensing 2 (publication) and Licensing 3 (licences follow the layer).
Not here: where the manifest schema lives after `isometry-campaign` retires, which is `campaign-retirement`'s question (this plan waits on it); the session carrier packs are distributed over, which is `session-network` (K0, K1); the stylesheet-versus-voxel appearance ruling, which is `board-on-isometer`; the sim's generator and pack-requirement semantics (a pack displaces generated content where the world meets its requirements), which are `sim` and `vtt-overlay` V2.

## Constraints

- **Packs made to be shared declare `license` and `credit`.** `credit` is the verbatim attribution the licence asks to travel with the content. Bundled packs are enforced by the test `every_bundled_pack_declares_its_terms` (`crates/isometry-system/src/tests/packs.rs`); a pack a user adds from their own roots may still declare nothing.
- **Example packs**: `core` ships and is the only pack the desktop host loads at boot (`crates/isometry-genet/src/boot.rs:25`); `demo` and `watchtower` are fixtures that tests and the watchtower self-test load.
- **A ruleset ships when its licence lets the wing use it without legal or community objection, checked per product** (doctrine Licensing 1). Qualifying: the 5e SRD (CC BY 4.0), Pathfinder 2e (ORC), Lancer (with attribution), and Ars Magica 5th Edition's open text (CC BY-SA 4.0, with Atlas Games' credit line; anything shipped from it is share-alike). Daggerheart does not qualify. An open text that informs the sim's design is a reference: nothing is copied into the sim. Mark, 2026-10-22: "Named profile, licence per product".
- **Assets**: original assets are CC BY-SA 4.0 with an attribution entry; imported assets keep their own licences and are recorded (doctrine Licensing 3).
- **The authoring tool stays private and terminal-only.** `ContentPackManifest` and its validator remain the real schema and `GeneratorPack::load` the loader. The tool changes no sim authority, grant, replication, catalogue admission, wire or domain hash. Authored manifests stay in their original TOML or externally supplied JSON (wing rulings 623, 624); unknown content stays in source; no new product identity encoding.

## Phases

### CP1. The VTT shows a pack's declared terms
The VTT surfaces a pack's declared `license` and `credit` to the table, and decides what an undeclared pack shows. Where it shows (pack picker, compendium, campaign-open screen) is open.
- Done when: not stated; owed.
- Needs: `campaign-retirement` (where the manifest lives); the authoring commands question below
- Rulings: -
- From: `crates/isometry-campaign/src/content/pack.rs:104-140`; `crates/isometry-genet/src/boot.rs:25`

### CP2. Authoring-tool toolchain residue
`cargo` 1.97.1 cannot resolve the workspace since Eidetic's chartulary manifest overrides a workspace dependency's `default-features` (a cargo 1.98+ form) while declaring `rust-version = "1.97.1"`; the plan called it "outside this plan". The authoring gates ran rustc and rustdoc 1.97.1 under cargo 1.99.1. A package-wide `-D warnings` run also exposed an existing `needless_borrow` in `crates/isometry-system/src/sys/generator.rs`, left unchanged and not re-checked.
- Done when: not stated; owed.
- Needs: an Eidetic supplier fix; the shared MSRV policy (mere's family composition thesis brief)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:30-33,195-200`

### CP3. Appearance preview and a real tileset
Two residues split out of the campaign-packs plan in the 2026-08-08 extraction: a pack appearance preview and a real tileset (shared with the gameplay roadmap's residue). They depend on which theory of appearance the VTT takes.
- Done when: not stated; owed.
- Needs: `board-on-isometer`'s owed stylesheet-versus-voxel appearance ruling
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-08_extracted_receipts.md:24-26,32-34`

### CP4. Rulesets that may ship
Mark's ask to Massif Press on Discord (Lancer is Massif Press's) is not done, and it is Mark's to make. Doctrine Licensing 1 lists Lancer as qualifying "with attribution"; the ask is what makes the attribution and any further terms concrete. Each ruleset's licence is checked per product before it ships in a pack, and its `license` and `credit` are declared in the pack manifest (CP1). The VTT's list of editions to play is `vtt-overlay` RS.
- Done when: not stated; owed.
- Needs: Mark (the ask); CP1
- Rulings: 40, 522, 541 (doctrine Licensing 1)
- From: `mesocosm/design_docs/archive_docs/2026-10-28/2026-09-22_sim_plan.md:864-869`

### CP5. Publishing `isometry-system`
The authoring tool's optional git dependencies (`mere-source-edit` on branch main, and the `merely-made/taplo` fork by revision) are outside `isocosm-vtt`'s closure but would block publishing `isometry-system` itself. Nothing is owed unless `isometry-system` is ever published.
- Done when: not stated; owed (only if `isometry-system` is to be published).
- Needs: `campaign-retirement` R4
- Rulings: 917 (publishable crates)
- From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:76-78,174-183,274`

## Deferred and parked

- **Pack distribution over Stickleback**: waits on K0 and K1 and "does not drive them". Also the distribution half of the worldbuilding residue is not separately tracked. Waits on `session-network` K0, K1. From: `archive_docs/2026-10-28/2026-08-08_stickleback_migration_plan.md:55-57`; `archive_docs/2026-10-28/2026-08-08_extracted_receipts.md:24-26`.
- **A headed authoring surface**: no GUI; "GUI adoption" and broader product certification are not claimed. Durable history, crash-recovery guarantees and a serializable handle API are outside the pass (history is per process, bounded, default text-only). Waits on a GUI consumer, and on Knot or Woodshed as a "mature second consumer" that "can establish the next lifecycle need independently of Isocosm's development schedule". From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:47-51,76-78`.
- **Save limits not qualified**: the final content check and rename is not compare-and-swap against an uncooperative writer; directory-fsync durability, ACL and xattr preservation and nonlocal filesystems are unverified; a lease from another host, or a PID reused by a live process (after a reboot, say), needs manual inspection and removal; when a third saver arrives while another reclaims, the restore step can still leave two holders; symlink manifests are inspected but not replaced. Waits on no plan; picked up if the tool is adopted beyond private use. From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:111-117,210-212`.
- **Publishing the taplo fork**: the fork is not registry taplo 0.14 and is not on crates.io; upstream master's 2025-05 to 2026-07 commits are not merged and upstreaming would rebase; the fork's formatter keeps comments and values in multiline inline tables but does not lay them out. Shared API extraction beyond the small `mere-source-edit` operation stays deferred. Waits on `campaign-retirement` R4 if `isometry-system` is ever published (CP5). From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:174-183,274`.

## Questions for Mark

- **Authoring commands for terms**: should `pack-author` get structure-aware commands for manifest-level `license` and `credit`? The existing `set_generator_field` sets only `id`, `name`, `default_args`, `script` and `fixtures`; the new keys are reachable only by ordinary text edits, and `check` returns the typed manifest (blank terms refused by `validate`). The example packs' declared terms are asserted by the bundled-packs test. Wing rulings 623, 624 keep manifests in their original TOML or external JSON. From: `archive_docs/2026-10-28/2026-10-17_private_pack_authoring_plan.md:145-172`.
- **What an undeclared pack shows**: the VTT must decide what the table sees for a pack that declares no terms (a user-added pack may declare none), and where declared terms appear. Options named in the old plan: pack picker, compendium, campaign-open screen. From: `crates/isometry-campaign/src/content/pack.rs:104-140`.

## Findings

- 2026-10-28: nothing outside tests reads `license` or `credit`; the only use is the example-pack assertion in `crates/isometry-system/src/tests/packs.rs`.
- 2026-10-28: `ContentPackManifest::validate` refuses a blank `license` or `credit` (`BlankTerms`) in `crates/isometry-campaign/src/content/pack.rs`; the bundled packs under `crates/isometry-system/examples/packs/` are `core`, `demo`, `watchtower`.
- 2026-10-28: the workspace toolchain is pinned to 1.99.1 (`rust-toolchain.toml`), so CP2's 1.97.1 resolution failure matters only to a consumer on the older cargo; the `needless_borrow` claim was not re-checked.
- 2026-10-28: the old campaign-packs decision 12 (spread as a DM-authority operation) is superseded by rulings 154 and 243.
- 2026-10-28: the pack-authoring plan's phases P1 to P8 are landed and are not carried; only its limits and deferrals are.

## Progress

- 2026-10-28: cut from the private pack authoring plan, the extracted-receipts residues and the pack lines of the campaign retirement plan.
