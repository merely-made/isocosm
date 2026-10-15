# The sim host: one embeddable core under three uniform product hosts

**Date:** 2026-10-14

**Status, 2026-10-14:** in progress. Opened by rulings 884, 892 and 895;
carries 887's first milestone; forks settled by 896 to 898. H0 and H1
landed; H2 assessed and its forks ruled (899 to 903).

## Why

A runtime check on 2026-10-14 found no head that plays. A default Mesocosm
launch refuses a generated world without 875's witness. The last headed save
(2026-09-08) no longer parses. Only the dev bench boots: `--bench --frames
240` rendered and captured, `RESULT ok`, receipt in
`Code/testing/isometry/review-2026-10-14/`.

Each product founds, drives and saves the sim its own way, and each has two
to four host paths. Ruling 884 asks for three uniform product hosts over one
embeddable sim host doing what all of them need: world founding, live keys
and pointer, and save import. Ruling 892 places that core in `isocosm-host`,
as a library. Ruling 885 keeps generated worlds behind 875's witness. Under
888 to 891 that witness comes from the generator, and the A1 lane builds it.

## Findings, 2026-10-14

| Product | Founding | Drive | Save | Clock |
|---|---|---|---|---|
| Mesocosm | `mesocosm_runtime::Founded::generated` (875-gated) | `Runtime`: envelope queue translated to native commands, envelope trace | `GameSave` (Game8) | wall-clock `Clock` |
| Eponym | `eponym-play/src/founding.rs` | `eponym-play` session and the client keymap | Game13 | none |
| VTT | native input files, by hand | `isometry-campaign/src/native` calling `command` directly | muniment campaign, JSON map | none |
| `isocosm-host` | none; resume only | transactions over `World` | `WorldSave` (native save plus opaque per-mode sections) | none |

- **The window side is already shared.** `isomere::host::Assembly` fills
  cambium's host hooks once: viewport producers, capture, the error line and
  the frame profile. The VTT (`isometry-genet`) and Eponym (`session`) use it.
  `mesocosm-genet` runs its own winit/wgpu loop. `isocosm-host` calls
  `cambium_genet_winit_host::run_windows` directly.
- **`isocosm-host` depends on the products.** Its `mesocosm`, `vtt` and
  `eponym` features pull in `mesocosm-genet`, `isometry-campaign` and
  `eponym-client`. For a core that products depend on, this direction must
  invert. Its default half is about 450 lines: `world.rs`, `save.rs`,
  `binding.rs`, `native_attention.rs`, `access.rs`, `stream.rs`,
  `file_save.rs`, `spatial.rs`. That half is the seed.
- **The generic parts of Mesocosm's `Runtime`** are the envelope queue, the
  translation seam, the pacing `Clock`, the trace and the save. Mesocosm's own
  parts are the interim loop (`directing::interim`), review, succession and
  glyphs.
- **Games bypass the overlay contract.** 80 Mesocosm, 76 Eponym and 134 VTT
  files import `isocosm::` directly. The core is where the contract becomes
  the real boundary.

**H2 assessment, 2026-10-15.**

- **An endpoint is four traits** in `graphshell-endpoint` (mere `0b9b7a9b`):
  `ProjectionCatalog::describe`, `ProjectionSource::snapshot`,
  `PresentationSource::resource` and `IntentSink::invoke`. A snapshot is a
  Scenograph `SceneSnapshot` plus a presentation manifest that binds scene
  instances to offers: portable card, native glyph, image, editable text.
  An advertised action carries an optional input form and an effect class:
  `Curation`, `DomainTruth` or `ExternalEffect`.
- **Inspection maps onto that directly.** Sites are scene instances, routes
  are relations, and individuals sit under their sites. Each instance gets a
  portable card (counts, accounts, traits, parts). Stepping and founding are
  `DomainTruth` actions, examining a site or body is `Curation`, and saving
  is `ExternalEffect`. The VTT's `isometry-graphshell` (576 lines) is the
  in-repo pattern: a stdio binary over `serve_basic`, tested through the
  traits.
- **No live viewer mounts an arbitrary endpoint.** The Graphshell port's
  `g4_sessions` mounts endpoint processes (`sessions::mount_endpoint_processes`)
  but writes a static HTML session receipt, which renders cards and
  actions. Turnstone hosts an endpoint process live, but only Knot's
  (`src/knot_authoring.rs`). Knot and Cleromancy prove theirs through
  `LocalCarrier` tests and receipts.
- **The desktop shell's product panels** are about 1,400 lines in
  `host/src/ui/`. Generic inspection (sites, crowds, bodies, parts, step,
  examine) is the endpoint's. Mesocosm checkpoints, Eponym successors and
  the VTT's journey, overmap, knowledge, adventure, authoring and downtime
  panels belong to product endpoints under 896. Nothing plays them today.

- **Society and feature unification, 2026-10-15.** The native-tests lane
  puts the society layer behind an `isocosm` feature `society`, on by
  default; Mesocosm opts out with `default-features = false` (893, 894).
  Cargo unifies features across one build graph, so once a Mesocosm crate
  depends on `isocosm-host`, `isocosm-host` must take `isocosm` with
  `default-features = false` and pass `society` through a feature of its own
  that the VTT, Eponym and host builds enable. Check with
  `cargo tree -e features -i isocosm --workspace` in `mesocosm/`.

## Phases

Forks settled by 896 to 898: the inspection shell becomes a Graphshell
endpoint, bench worlds prove the head before 875's inputs, and all three
product hosts sit on isomere's `Assembly`.

**H0. The core library.** `isocosm-host` gains a library target holding
`World`, `WorldSave`, attention binding, file saves and streams, with no
product crate in the library's graph. The desktop shell stays the package's
binary for now; nothing depends on the library yet, so there is no cycle.
Done when the library builds and tests without any product feature; the
binary builds with all three features and its existing tests pass; and
`--validate-save` behaves as before.

**H1. Founding.** The core founds a `Session` from a product's founding
request: preset, seed, population and the generated world's 875 witness when
there is one. Under 885 a head refuses a generated world without the
witness; under 897 the bench, a dev instrument, founds its worlds.
Done when one test refuses a witnessless generated founding for a head; the
bench founds through the core to the same state hash as its existing path;
and the founded world round-trips through `WorldSave`.

**H2. The Graphshell inspection endpoint (896, 899 to 904).** Four steps;
under 904 step 3 (the shell's retirement) runs before step 2, so the panels
port onto the core's `World` without a package cycle:

1. The core serves a Graphshell endpoint from an `endpoint` module behind a
   feature in `isocosm-host` (901). Sites, routes, crowds, bodies and parts
   are scene instances with portable cards. Founding and stepping are
   `DomainTruth` actions, examining a site or body is `Curation`, and saving
   is `ExternalEffect`. The `isocosm-host` binary serves it over stdio,
   beside `--validate-save`.
2. Every desktop-shell product panel moves to a product-owned endpoint
   (900), with no new `isometry` names. The VTT's journey, overmap,
   knowledge, adventure, authoring and downtime panels go to `isocosm-vtt`
   (902). Mesocosm's checkpoint and board answers go to an `endpoint`
   module in `mesocosm-runtime`, and Eponym's successors to one in
   `eponym-play` (903), each behind a feature.
3. The desktop shell (`ui/`, `desktop`, `products`, `viewport`, `cli`,
   `mode_sections`, `shell_tests`) and `isocosm-host`'s product features
   retire, so products can depend on the core without a cycle.
4. When the capabilities being lifted out of Graphshell's reference host
   land (899), the endpoints adopt them for live viewing.

Done when each endpoint's projections, cards and actions pass
`LocalCarrier` tests against a bench world, with intents accepted and
refused as the panels did; a `g4_sessions` receipt mounts the core and
product endpoints and renders them; and `isocosm-host` has no product
dependency left. The four shell tests that already fail on main go with
the shell; any behaviour they meant to cover is carried by an endpoint
test or recorded as dropped.

**H3. Drive and pace.** The core's driver owns the envelope queue, the
wall-clock pacing, the per-advance step ceiling and the envelope trace. A
product supplies its translation from overlay envelopes to native commands
and its own loop hooks. Mesocosm's `Runtime` is re-expressed on it, and its
interim loop stays Mesocosm's.
Done when `mesocosm-runtime`'s tests pass unchanged; a recorded Mesocosm
trace replays to its recorded hash through the core; and a planted fault in
the translation seam is caught.

**H4. Live input.** Keys and pointer arrive from the window host. The core
dispatches them through a product keymap to envelopes, and resolves pointer
picks against the product's viewport leaf.
Done when `mesocosm-genet`'s E, Space, click, Q, right-click and S, and the
checkpoint and board keys, reach the session through the core; a headless
input test drives them; and the bench scenarios still pass.

**H5. Saves and import.** `WorldSave` is the one envelope. The core imports
a Mesocosm `GameSave`, and later Eponym Game13 and the VTT campaign, into it
with the product section kept as opaque bytes.
Done when a save written by the Mesocosm head loads in the core and resumes
to the same hash, and an unknown section survives a round trip untouched.

**H6. The Mesocosm head on the core and `Assembly` (898).** `mesocosm-genet`
founds, drives, takes input and saves through the core, and its window moves
onto `Assembly`. The terrarium section becomes a viewport producer (the
existing `borrowed_section::SectionProducer` is the start), and its chrome
becomes Cambium views.
Done when the head boots on a bench world with a captured frame; `--watch`
resumes a save written through the core; frame pacing and pointer picking
in the viewport leaf are measured against today's loop; and once 885
resolves (the A1 lane's 890 capture path passes the joint witness) a default
launch founds a generated world. For 887, a lineage reaches the epoch
boundary and the trait board, captured.

**Later.** Eponym's `session` on the core (founding a generated world, asks
through the UI, succession in the client), then the VTT over the sim
(in-app campaign founding from a seeded draw). Each gets phases here when it
starts.

## Progress

- **2026-10-14, H1 landed:** `isocosm_host::founding::found(genesis,
  execution, purpose, offered)` founds the core's world. A product builds
  its own `Genesis`; the core owns the rule. A `Purpose::Head` founding a
  generated genesis (`genesis.founding` set) needs an `Offered` witness and
  admission whose `genesis_digest` names this genesis and whose
  `Admission::validate` passes against the witness; `Purpose::Bench` (897)
  and a genesis naming no generator need neither. *Reading, not ruled:* the
  plan's "founding request: preset, seed, population" became "the product's
  genesis", since presets are product data. Four tests: a head's refusal
  without a witness; the bench's own inputs (seed 10975940, 256, 6 sites, 4
  lineages) founded through the core match `Session::new` directly, at tick
  0 and after five ticks; a genesis naming no generator founds without a
  witness; a founded world round-trips through `WorldSave`. A planted fault
  disabling the head gate was caught and exact bytes restored. Library 30
  passed, 5 ignored; the all-target, all-feature check is clean.
  Two parts wait: the bench's own call site (`mesocosm-genet`
  `app/bench/sim.rs`) moves onto the core after H2 removes the package
  cycle, and the admission-binding branch (accepting a binding admission,
  refusing another genesis's) is unexercised until a real admitted witness
  exists, which A1's 890 capture path brings.
- **2026-10-14, H0 landed:** `isocosm-host` has a library (`host/src/lib.rs`)
  holding `access`, `binding`, `file_save`, `native_attention`, `save`,
  `spatial`, `stream`, `window` and `world`, with no product crate named in
  its sources. `World` lost its three shell-only methods for generic section
  access (`with_sections`, `read_sections`, `product_sections`); the shell's
  `mode_sections::Games` and `products::Submit` wrap them, and the Eponym
  handoff moved to `mode_sections::take_eponym`. The shared fixture is the
  library's `fixtures` module (feature `fixtures`); the shell's product
  tests live in `src/shell_tests/` and refuse to compile without it.
  Gates in the lane worktree: library tests with no features 26 passed and
  5 ignored; shell tests with all three products 9 passed, 4 failed, 2
  ignored; `cargo check --all-targets --all-features` clean of new warnings;
  `--validate-save` gives identical output to the unsplit binary on a gate
  save, a non-save and no arguments. The unsplit baseline at `6d6ac1df` runs
  the same 46 tests to the same 35 passed, 4 failed, 7 ignored, the four
  failing at the same lines, so H0 changed no outcome.
- **2026-10-14, finding:** four shell tests fail on main independently of
  H0, unseen because the certification gates compiled the host and ran none
  of its tests. Two refuse at `configure the explicit played loop before
  stepping` (`shell_tests/products.rs:65`, `reading.rs:111`), two at viewport
  registry assertions (`viewport.rs:21`, `:50`). The shell they test retires
  at H2 (896); they are recorded, not repaired.
- **2026-10-14:** plan written from the runtime check and the code read
  above; rulings 884 to 895 recorded. Forks settled by 896 to 898; the
  Graphshell endpoint (H2) now removes the package cycle, so no shell
  package is needed.
