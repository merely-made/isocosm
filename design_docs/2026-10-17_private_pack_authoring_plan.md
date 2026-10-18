# Private generator-pack authoring

**Date:** 2026-10-17, updated 2026-10-18. **Status:** private terminal consumer
and shared edit/undo adapter implemented; receiving the published
[`mere-source-edit` supplier at `70deef2c`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f)
without local overrides is qualified in the working tree at `183a2f67`.
The consumer is not committed or pushed. Runtime/session ownership is unchanged.
P5/P6 receipts retain their original local-supplier baselines; P7 records the
Git-only receiving checks. Git history records consumer publication separately.

## Ownership and policies

The app-local `pack-author` command retains authored source and revisions.
`ContentPackManifest` and its validator remain the real schema; the existing
`GeneratorPack::load` remains the runtime loader. No campaign retirement work,
sim authority, grant, replication, catalogue admission, wire or domain hash is
changed. The tool emits machine JSON responses, but authored manifests remain
in their original TOML or externally supplied JSON format (wing rulings 623/624).
TOML 1.1 and full real `GenValue` decoding stay authoritative through the same
product dependencies; taplo diagnostics for unsupported grammar are advisory
when the authoritative decoder succeeds. Unknown content stays in source;
there is no new product identity encoding.

## Phases and done-conditions

- P1 complete: optional private batched Rowan 0.17 backend and source provenance. Normal
  builds do not depend on it; licensing and vendor file-layout exceptions are
  recorded. The tool feature is not a published fork distribution.
- P2 complete: retained immutable revisions, expected-base edit checks, source inspection,
  typed checks, undo and explicit save with external-change refusal, count-limited
  history, no implicit application admission. CLI commands exercise the actual
  consumer, not a copied stand-in schema.
- P3 bounded checks passed: real fixtures and full ABI, TOML 1.1/JSON compatibility, invalid draft saves,
  stale edits and external changes, undo, exact-byte fresh-process reopening,
  existing fixture execution after authoring, and default/feature compiler and
  focused regression gates. Record implementation and verification separately.

- P4 implemented and qualified (2026-10-17, initial baseline `1dcbe606`): private reviewable lock-preset
  insertion. Proposal/explicit edit/save separation, typed ownership,
  conflict/stale/index refusals and the real scratch-pack workflow pass.

- P5 implemented and locally qualified (2026-10-18, baseline `183a2f67`):
  route private edits and preset previews through `mere-source-edit`; keep the
  existing CLI JSON, parser/schema/layout checks, explicit saving and admission
  ownership. Done-conditions: host-stamp/receipt regression tests, default and
  authoring gates, real scratch-pack workflow, scoped lock diff and explicit
  unpublished-supplier limitation recorded below.

## Findings

2026-10-17 at `d91a8290`: `GeneratorPack::load` already accepts TOML 1.x and
external JSON, validates the real manifest before loading assets, and rejects
a directory containing both manifest formats. The app's native adventure-pack
admission is a different owner; this change does not enter that path. Campaign
retirement is active, so no new ownership is added to `isometry-campaign`.

The evaluated batching backend and raw receipts live in
`Code/testing/retained-dom-batch-20261016`; the architecture/evidence home is
`mere/design_docs/2026-10-14_retained_document_resource_brief.md`. No optimization
or canonical-JSON domain hash is introduced outside the private syntax tool.

## Progress

- 2026-10-17: fetched and fast-forwarded clean Isocosm main, inspected active
  retirement/format rules, and scoped the terminal consumer to optional system
  tooling. Qualification pending.

## Using the private command

Check a pack with the real schema, without changing it:

```sh
cargo run --locked -p isometry-system --features pack-authoring --bin pack-author -- crates/isometry-system/examples/packs/demo --check
```

Omit `--check` to open a terminal session. It prints one ready record, then
accepts one JSON command per line and emits one JSON response. This protocol
is machine interchange; the source manifest is never transcoded or rebuilt
from those responses. Example commands (byte offsets, current base required):

```json
{"op":"read","text":true}
{"op":"edit","base":1,"edits":[{"start":0,"end":0,"text":"# local note\n"}]}
{"op":"check"}
{"op":"inspect"}
{"op":"undo","base":2,"target":1}
{"op":"save","base":3}
{"op":"quit"}
```

`check` returns the real typed manifest and declared generator choices; it
opens no assets and admits nothing. `inspect` exposes ordered source ranges
only when the older taplo grammar can index the complete TOML document.
TOML 1.1 index errors and JSON's absent index are advisory; the existing product
decoder/validator remains authoritative. `read` preserves foreign content that
is not represented in the typed manifest. Failed commands return `ok:false`
and the session continues; one-shot `--check` exits unsuccessfully on refusal.

Edits and undo publish new revisions; no implicit save or application occurs.
The session retains the latest 32 revisions. Reopening starts a new local
revision sequence; this is not durable history or a serializable handle API.
UTF-8 byte ranges must end at character boundaries; overlap and same-start
batch edits are refused. The authoring tool has a 2 MiB source and 64 lexical
TOML bracket/brace nesting limit; these are tool limits, not new loader rules.

Explicit `save` also saves invalid drafts. Such a file will fail the existing
loader until repaired. Files retain their original TOML/JSON format, comments,
foreign content, literal strings and line endings. Saving checks for external
changes and a newly added second manifest; conflicts require explicit reopen.
A cooperating writer's `.isometry-pack.toml.author.lock` (or JSON equivalent)
blocks another save. The tool preserves file permissions, syncs a temporary
file, then renames it in the same directory. Symlink manifests can be inspected
but are not replaced by save. A failure before replacement leaves the target
and saved marker intact.

The final content-check/rename gap is not compare-and-swap against an
uncooperative writer. Directory-fsync durability, ACL/xattr preservation,
stale-lease recovery after crashes and nonlocal filesystems are not qualified.
A crash can leave the lease; inspect it before manually removing it. The
single-owner command loop serializes edits and saves; there is no background
save to mark a newer revision saved accidentally.

## Reviewable preset insertion

`propose_lock_preset` is a private read-only command. Example JSON-line request:

```json
{"op":"propose_lock_preset","base":1,"generator":"forge_item","preset":{"key":"quality","label":"Heirloom quality","value":{"type":"text","value":"heirloom"}}}
```

It returns the document, expected `base`, UTF-8 `edits`, full proposed text in
`preview`, the preset and target, and `applied:false`/`admitted:false`. Review the
proposal, then submit its `base` and `edits` using the existing `edit` command.
Saving remains a separate explicit `save`. An intervening edit makes the old
proposal's base stale. The operation accepts the real `GeneratorLockPreset`
and full `GenValue` schema, validates the intended manifest, and reparses the
proposed source to require exact typed equality with that manifest.

The supported first slice requires a valid typed TOML manifest with a complete
source index, canonical unquoted `[[generators]]` and
`[[generators.lock_presets]]` headers (header whitespace is accepted), a unique
local generator ID and a new valid preset key. Generator/preset descendant headers and alternate/inline/quoted layouts,
partial TOML 1.1 indexes, JSON, missing or ambiguous targets, invalid presets
and conflicting keys refuse without mutation. Some other layouts may also
refuse if candidate validation cannot prove ownership; ordinary text edits
remain available. A proposal does not admit data. The shared edit-plan kernel
does not own these format-specific rules.

## Backend provenance and distribution

`vendor/taplo` retains the upstream MIT notice and a reviewable patch from
published taplo 0.14 to Rowan 0.17 plus construction-time batching. It is an
excluded standalone workspace and an optional dependency of this command,
not a new shared API. Parser Rust file layout is preserved by six explicit
vendor-only line-ceiling exceptions; first-party files remain below 600 lines.
No counters or scratch hash format from testing is shipped. The root lockfile
keeps existing package versions; additions are the optional backend closure.

The fork is unpublished and is not registry taplo 0.14. Distributing/publishing
the authoring feature requires a qualified fork source; this private integration
does not claim the publishing lane is complete. The original probes remain in
`Code/testing`; no test artifact is a product admission or identity authority.

## Verification, 2026-10-17

[Qualification receipt](receipts/2026-10-17_pack_authoring/qualification.json)
and adjacent logs record:

- 68 tests pass with `pack-authoring`: 49 existing library tests, eight authoring
  unit tests, two campaign-assertion tests, two native campaign-pack tests, two
  fresh-process terminal tests and five watchtower tests. The final eight-test
  binary run additionally covers editing/saving TOML 1.1 with a partial index.
- 58 default-path tests pass; normal dependency-tree inspection finds neither
  taplo nor Rowan. The vendor is not a default workspace member.
- The backend's 101 inherited tests pass. Its source has no experimental counters;
  the patch and MIT provenance remain reviewable under `vendor/taplo`.
- Locked all-feature/all-target checking of `isometry-system` and a separate
  Rust 1.97.1 binary check pass on the product's pinned source closure.
- Scoped all-target Clippy passes. The new binary and integration test deny
  `clippy::all`. A package-wide `-D warnings` run exposes an existing
  `needless_borrow` in `src/sys/generator.rs:301`; it was left unchanged, along
  with dependency and existing sim/campaign warnings.
- Scoped rustfmt, changed-source line ceilings, authored-pack format policy,
  and `git diff --check` pass. Existing root lock package versions/sources are
  retained; the optional backend adds its closure without repinning suppliers.

The terminal tests exercise real pack authoring and distinct-process reopening:
invalid source is saved/reopened as a draft, repaired to the annotated pack,
inspected, undone/restored, saved and checked in a third process. The existing
`GeneratorPack::load` then loads it and runs the declared forge-item fixture.
A retained loaded pack and disk remain unchanged by unsaved/stale commands.
The authoritative TOML 1.1/JSON decoder accepts the same schema; modern syntax,
float/date extension values, and nested real `GenValue::Object/List` values
survive authoring. No probe-local schema or hash is used.

- 2026-10-17: P1–P3 implemented and bounded checks passed; runtime, schema and
  admission code left unchanged. Optional terminal integration only; no headed
  authoring surface or broader product certification claimed. Publishing the
  private fork, GUI adoption, durable history and crash-recovery guarantees
  remain outside this pass. Shared API extraction remains deferred.

## P4 qualification — 2026-10-17

The private insertion addition was initially qualified on `1dcbe606`. Seventy-one
feature-enabled package tests (including 11 binary tests), 58 default tests,
scoped all-target Clippy, Rust 1.97.1 binary compilation and formatting pass.
Existing library/dependency warnings remain separate; no package-wide strict
lint claim is made.

`Code/testing/pack-author-preset-20261017/REPORT.md` records the current binary's
59-command, five-process real-pack workflow. Proposal preview is read-only;
explicit `edit` matches the preview and leaves disk unchanged until `save`.
Stale proposals/applications, duplicate keys and valid TOML 1.1 with a partial
index refuse without mutation. The existing invalid-draft save/reopen/repair,
external-conflict export/reopen/reapply and actual loader/fixture steps pass.

Focused tests additionally cover middle/last generators, CRLF and final-line
comments, nested real GenValue values, quoted/newline/Unicode labels, missing
and duplicate targets, invalid presets, JSON/quoted layouts and unknown preset
descendant tables. The latter refuse because typed equality alone cannot prove
that unknown content remains attached to the original array element.

No loader, schema, runtime, save policy, dependency or admission changes are
made. This completes only the small private slice; shared API extraction stays
deferred. A mature second consumer such as Knot or Woodshed can establish the
next lifecycle need independently of Isocosm's development schedule. Artifacts
remain in testing; Git history records publication separately from qualification.

Publication recheck, 2026-10-17: fetched and fast-forwarded to `b918c8a0` without
changes to the qualified private source. All 71 feature-enabled tests pass;
the rebuilt CLI/example repeat the 59-command, five-process workflow with actual
fixture execution. Logs and receipt remain under
`Code/testing/pack-author-preset-20261017/publication/`. Shared API remains deferred.


## P5 shared edit-plan adapter — 2026-10-18

This phase supersedes the earlier shared-API deferral for the small source-edit
operation. `Session::source_stamp` supplies the canonical-root manifest path,
open-instance counter and current revision to `mere_source_edit::Proposal`.
`Session::edit` prepares and checks an application in its serialized command
handler, then uses the existing `publish` path: preflight limits, parser/schema
diagnostics and immutable history publication stay local. Shared changed ranges
map into the unchanged flat CLI receipt (`document`, `base`, `result`,
`operation`, `ranges`). Revision exhaustion retains the legacy error wording.
Undo and all filesystem save/conflict behavior remain local and unchanged.

Preset insertion now gets both preview and serialized edits from that same
validated plan. The real expected-manifest validation, reparse/equality check,
batched TOML index, generator ownership and unknown-descendant/layout refusals
stay in Isocosm. The shared helper has no typed application state. Its internal
stamp and exact-source guard do not change the private protocol into a durable
cross-process proposal format; clients still submit the existing session's
expected `base` and text edits.

The canonical new dependency is optional `mere-source-edit` 0.1.0 from
`https://github.com/merely-made/mere.git`, `branch = "main"`, enabled only by
`pack-authoring`. Qualification uses the explicit config
`Code/testing/shared-source-edit-20261018/isocosm/local-source-edit.toml`,
patching **only** that new package to
`/Users/markik/Code/repos/mere/crates/system/source-edit`. Existing Mere supplier
pins/patches and other package versions/sources are untouched. Normal runtime
dependency traversal contains none of `mere-source-edit`, Taplo or Rowan.

Seventy-four authoring-enabled package tests (14 binary tests), 58 default tests,
all-feature/all-target package checking, scoped all-target Clippy, a separate
Rust 1.97.1 binary check and formatting pass with the explicit patch. Three
adapter tests cover host identity/reopen binding and source guards, exact legacy
receipt JSON/strict edit input and transactional overflow refusal; the existing
parser, schema, layout and save tests pass unchanged.

`Code/testing/shared-source-edit-20261018/isocosm/REPORT.md`, `checks.json` and
`workflow/qualification.json` retain commands, process exit statuses, source and
binary hashes and the real scratch-pack transcript. The workflow again passes
59 commands across five processes: reviewed insertion and explicit apply/save,
stale/duplicate/partial-index refusal, unknown-content preservation, invalid-draft
save/reopen/repair, external-conflict recovery and actual loader/fixture execution.
Original real-pack files and scratch assets are hash-checked unchanged.

Unpatched remote resolution was actually attempted and fails because the new
package is not yet on Mere main. The lockfile records the locally patched new
package without a Git source; it is an assembled-source qualification, not a
portable remote lock or publication receipt. After supplier publication the
owning integration must regenerate only this package's remote lock entry and
rerun unpatched checks. No commits, pushes, GUI qualification, crash-recovery
claims, runtime admission changes or broader supplier repins are part of P5.

## P6 duplication removal — 2026-10-18

Edit and undo now share one `apply_edits` path and one publication boundary.
Undo supplies a full-source replacement; it no longer constructs range receipts
by hand. Publication takes the checked result revision and ranges from the
shared application instead of incrementing the revision a second time. Format
preflight and syntax construction still finish before current/history mutation.
Save, schema, admission, stale-base precedence and CLI receipt shape are unchanged.

Seventy-five feature-enabled tests (15 authoring), 58 default tests, scoped
all-target Clippy, Rust 1.97.1 binary checking and build pass with the same
explicit only-new-package local override. The 59-command/five-process real
workflow passes, and all 64 ready/command records match the prior shared-adapter
workflow after scratch-path normalization. New controls cover undo's exact range
receipt, stale and no-op undo, and both edit/undo revision-exhaustion refusal
without publication or save. The report and logs remain in
`Code/testing/source-edit-dedup-20261018/isocosm/`. Publication is still separate;
this pass does not commit, push, or change source dependencies.


## P7 published supplier receiving checks — 2026-10-18

The supplier is published on canonical Mere main at
[`70deef2c8cf3d3410ae35b496a9396514070b24f`](https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f).
The optional manifest remains Git `branch = "main"`; no URL spelling trick,
local patch/config, environment override or other supplier repin is used.
Scoped `cargo update -p mere-source-edit` could not select the prior locally
patched package after removing the explicit override. A normal unpatched
`cargo check -p isometry-system --features pack-authoring --bin pack-author`
generated the remote lock entry instead. Its source is
`git+https://github.com/merely-made/mere.git?branch=main#70deef2c8cf3d3410ae35b496a9396514070b24f`.
Every existing lock package/version/source remains unchanged; the only existing
dependency-list change is `isometry-system`'s new optional edge. No broader
source coupling was required.

Receiving checks pass without overrides: 75 feature tests (15 authoring),
58 default tests, all-feature/all-target package checking, scoped all-target
Clippy, Rust 1.97.1 binary check, build, formatting and diff checks. Resolved
Cargo metadata points the shared package into Cargo's Git checkout at the
published source; no filesystem patch provides it. The default normal dependency
tree still excludes `mere-source-edit`, Taplo and Rowan.

The rebuilt Git-backed CLI repeats the real scratch-pack workflow: 59 commands
in five processes, exact save/reopen, invalid-draft repair, explicit proposal/
edit/save, stale/duplicate/index refusals, external-conflict recovery and actual
fixture execution. All 64 ready/command records match the previous deduplicated
workflow after replacing only scratch-directory paths. New source/binary hashes,
commands, metadata and receipts are retained under
`Code/testing/source-edit-publication-20261018/isocosm/`; earlier receipts are
unchanged. This consumer commit records the qualified Git-backed adapter.
Publication does not extend the evidence to a GUI or additional admission
authority.
