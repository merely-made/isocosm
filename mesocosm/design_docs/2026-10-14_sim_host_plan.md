# The sim host: one embeddable core under three uniform product hosts

**Date:** 2026-10-14

**Status, 2026-10-16:** in progress. Opened by rulings 884, 892 and 895;
carries 887's first milestone; forks settled by 896 to 898. H0, H1, H2 and H3
landed (H2's forks ruled 899 to 904); H2's step 4 waits on the lifted
reference-host capabilities (899). 2026-10-16: H4 (954 to 957) and H5
landed. The ordinary Mesocosm window now runs on Assembly, renders through
the section producer and resumes core saves. Its native controls and replay
pass. H6 remains in progress: default launch waits on A1's joint witness.

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

**H3 assessment, 2026-10-15.**

- **Mesocosm drives one saved loop through two doors.** The owned
  `mesocosm_runtime::Runtime` (`runtime.rs`, 422 lines, with `runtime/`)
  keeps its own `Interim`, whose `Session` is not the core's `World`, plus
  the `Clock`, a `VecDeque` of envelopes, an in-memory trace of each
  envelope with its outcome, a step ceiling (default 8) and the review,
  checkpoint and glyph state. The borrowed door (`borrowed::submit_game`,
  `continuation::advance_loop`) runs the same saved loop over a host's
  session, and is what Mesocosm's endpoint uses over the core's `World`
  section. `runtime/loop_state.rs` says it: "The owned driver uses the same
  saved loop as the borrowed product door." The borrowed door refuses a
  save with an owned queue in it ("saved owned queue requires its original
  owned driver").
- **The generic parts are small.** `Clock` (`clock.rs`, 172 lines with six
  tests) turns integer microseconds into whole steps in rational form, so
  60 Hz never drifts, and a cap defers steps rather than dropping them. The
  queue, the trace and the ceiling are about 60 lines of `Runtime`. The
  translation seam is `runtime/translate.rs` and the shared
  `translate/direct.rs` (a nudge to `Command::Nudge`, an act to
  `Speciate`, dev intents native where native has a command); the loop's
  hold and answer logic is Mesocosm's own.
- **Founding is a product step on top of the core's.** `Interim::found`
  creates the session, runs deep time until the played lineage has a
  habitable site, then hands the player over; H1's `found` stops at the
  session.
- **Pacing differs across the products.** Mesocosm paces rounds by wall
  clock through `Clock`. Eponym's session client paces motion at 60 Hz
  with `Instant` and drops its backlog after eight catch-up steps
  (`eponym-client/src/bin/session/actions.rs`); its native clock advances
  a tick per recorded act. The VTT has no wall clock: native time moves by
  consented downtime. The core's endpoint steps one tick on request.
- **The driver is barely tested where it lives.** `mesocosm-runtime` runs
  31 tests (21 unit, 6 ecology offer, 4 endpoint): the clock, the effect
  experiment, tactile and the voxel profile. None drives `Runtime`'s
  queue, holds, step ceiling, trace, save or resume. `mesocosm-genet` holds
  28 tests, some through `Runtime`, and the head's bench scenarios. No
  Mesocosm trace is recorded in today's format: the last headed save
  (2026-09-08) no longer parses.
- **Users.** `mesocosm-genet` drives `Runtime` (`queue`, `advance`, `step`,
  `trace`, `submit_loop_input`, `save_game`, `receipt`,
  `resume_game_with_mode`); `mesocosm-views` reads it.

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

**H3. Drive and pace (925 to 928).** The core gains a driver over its
`World`. A product trait names the product's envelope type and supplies
its translation into native commands, its holds and its round (926); the
driver owns the queue, the envelope trace, the per-advance step ceiling and
the pacing: the core's `Clock`, Mesocosm's moved, for wall-clock products,
and an explicit step for the rest (927). Mesocosm's `Runtime` becomes a thin
product layer over the core's `World` and its `mesocosm` section, one door
for the head and the endpoint (925); `Interim`'s founding stays a product
step after H1's `found`, and the interim loop stays Mesocosm's. Three steps
(928): (1) pin today's `Runtime` with tests of its queue order, holds and
answer draining, step ceiling, trace outcomes, and save and resume, passing
on today's code, and record an envelope trace from a generated world;
(2) the core's driver and clock; (3) `Runtime` on the core.
Done when the pinning tests pass unchanged before and after; the recorded
trace replays through the core's driver to its recorded hash; a planted
fault in the translation seam is caught; `mesocosm-runtime`'s and
`mesocosm-genet`'s tests pass; and the endpoints' tests and the g4 receipt
still pass. Eponym's motion adopting the clock, and the VTT on the
explicit step, come later.

**H4. Live input.** Keys and pointer arrive from the window host. The core
dispatches them through a product keymap to envelopes, and resolves pointer
picks against the product's viewport leaf.
Done when `mesocosm-genet`'s E, Space, click, Q, right-click and S, and the
checkpoint and board keys, reach the session through the core; a headless
input test drives them; and the bench scenarios still pass.
*Forks ruled 2026-10-16 (954 to 957):* the core defines its own `Press` and
a keymap hook on the driver's `Product`; only keys that make an envelope go
through it, host-state keys staying the window's; the window still picks and
the core refuses a pick stamped with a stale world revision; the live-input
backlog cap (default 10) moves into the driver.

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

**H7. `isonetry`, remade as the family's session layer (923, 924).** One
peer hosts the core's `World`; other peers send intent envelopes and receive
the accepted log, so a world is played from several machines in each of the
three games. It starts with an assessment, whose forks go to Mark before
code: what Graphshell's network carrier (`graphshell_endpoint::network`)
already carries for the endpoints; Stickleback's joined spaces, which the
[Stickleback plan](../../design_docs/2026-08-08_stickleback_migration_plan.md)
adopts for campaign sync (K0 to K2); the stack's transport (murm,
`mere-transport`); how much of `isonetry`'s replication core (protocol,
host and client state machines, rolling log hash, iroh link) carries over;
and where its VTT-only parts go (`isocosm-vtt`, with the GM store and
proposals held in `isonetry` until V2 under 921). Draft done-condition, to
be set by that assessment: a world hosted on one peer accepts intents from a
second over the stack's transport through each product's endpoint, both
peers reach the same world revision and state hash, and the VTT's table runs
on it with the old DM-authority core retired.

**Assessment, 2026-10-16, source only:** the authority is already settled by
H7: one host accepts and orders intents. Adopting a multiwriter carrier does
not reopen that ruling. Current `isonetry` protocol 22, `HostSession` and
`ClientSession` supply useful sequencing, gap handling, request identities,
compatibility refusal and rolling log-hash behavior. Their concrete
`GameSnapshot`/`GameEvent` vocabulary, campaign store and table state remain
VTT-specific. `campaign_sync.rs` still assembles `LogSync` and `SyncedSpace`
directly, so Stickleback K0 is genuinely open. Under 921, the GM store and
proposal lifecycle stay in `isonetry` until V2; they must sit outside its
new family API.

At consumed Mere `0b9b7a9`, Graphshell's `NetworkCarrier` carries Chirograph
requests, responses and revision notices over an already admitted stream;
it owns neither admission nor game-log replication. Stickleback's
`JoinedSpace<E>` owns signed operation-log joining, publication, resync and
shutdown; Murm's consumed `b49df894` transport supplies its endpoint and
gossip. These are distinct seams. Current Mere `4523cb6` changes the carrier's
line-framing calls, while `joined_space.rs` is unchanged at those pins.

The assessment leaves two concrete forks before H7 code (923):

| Fork | Recommended first slice | Other defensible path |
|---|---|---|
| Accepted record | Publish the accepted native log suffix and atomic opaque section updates. A client applies accepted authority without running a product adapter to decide again. | Publish accepted product envelopes and replay them through a pinned adapter. This is smaller when envelopes are small, but every replica needs the exact product adapter and its compatibility contract. |
| Transport composition | Endpoint requests/replies use the admitted stream and Graphshell carrier; host-authored accepted records use Stickleback `JoinedSpace` for ordered catch-up and resync. One host remains the sole accepting writer. | Use the admitted stream for both requests and log catch-up first. This reuses less of Stickleback's existing operation-log lifecycle and leaves that convergence work in `isonetry`. |

A live `World` revision resets when its save is resumed; native state hash
alone also omits opaque product sections. Either record choice therefore
needs an explicit session/checkpoint identity, monotonically ordered record
sequence and parent digest, native-log cut, section digest, and recorded
revision at the checkpoint. A section deletion must be an explicit update;
unknown sections remain bytes. Reconnect installs one checkpoint and checks
the contiguous accepted suffix before exposing the new cut. A rejected
intent advances no accepted sequence. Pending client intent remains a
proposal until its host record arrives. Host authority and 921's interim
VTT ownership are already settled.

The first implementation gate should exercise two in-process peers over
serialized records: native plus section-only changes, unchanged bytes,
refusal, duplicate/gap/out-of-order delivery, checkpoint resume, and an
unknown section. The network gate then uses the actual consumed transport
and endpoint before each product and the VTT table claim acceptance. Both
peers must match accepted sequence, native hash, section digest and world
revision. These are proposed done-conditions for Mark's fork, not a new
protocol already accepted or implemented.
No H7 code or network receipt is claimed by this assessment.

**Later.** Eponym's `session` on the core (founding a generated world, asks
through the UI, succession in the client), then the VTT over the sim
(in-app campaign founding from a seeded draw). Each gets phases here when it
starts.

## Progress

- **2026-10-16, H6 ordinary window:** `Host::run` now always enters Assembly;
  the private legacy window/GPU loop and its frame/setup/receipt modules are
  retired. The existing section producer owns the viewport; Cambium views
  carry the trait board, checkpoints, hints, vitals and inspector, with the
  existing minimap leaf. Pointer coordinates come from the viewport leaf.
  Inspector selections validate against the actually presented body and
  expire on an unpresented revision. Presentation retains camera, follow,
  terrain height and dev focus independently of native world input.
  Ordinary founding now enters the core; an admitted generated witness loads
  its authenticated trajectory through the core without resetting its tick.
  The witness checks execution, genesis and admission before product setup.
  Nineteen runtime tests pass (bench, offer, pins and saves), and the native
  producer picking/selection test passes. The all-target compiler gate passes.
  `ordinary-controls-receipt.json` in
  `Code/testing/isometry/h6-2026-10-16/` records 51 frames, two captures and
  `RESULT ok`: pause, minimap/world pointer separation, actual body picking,
  I/L/U inspection and hint buttons. Its saved core replay lands on the
  recorded hash `7e8769106f020d48`. The ordinary watch run retains the earlier
  six-tick board hash `99c12f34e2e6eb94`. Source declarations and failed runs
  stay beside these receipts. One exploratory control run incorrectly
  expected a trait board after EndEpoch on the stock long-epoch world; it
  reached the legitimate loss checkpoint instead. Its artifacts retain the
  `end-epoch-loss-` prefix. The short-epoch board run remains the board receipt.
  Bench and resumed ordinary worlds play; generated default launch and the
  complete H6 done-condition remain open under 875/885 and A1.

- **2026-10-16, H6 bench slice, in progress:** `--bench-world` is an explicit
  `Purpose::Bench` founding, through the core before native `Interim` setup;
  it does not admit an ordinary generated head. `--bench-epoch N` gives this
  bench's boundary. The new `app/assembly/` uses `isomere::host::Assembly`,
  Mesocosm's existing Cambium vitals/board/checkpoint/dev views, and its actual
  `SectionProducer` on the present device. Keys reuse the core path; leaf
  clicks convert logical coordinates to the rendered target and carry its
  world revision. New input before presentation is refused. The executable
  needed an 8 MiB main stack on MSVC (configurable with
  `MESOCOSM_STACK_BYTES`) and a leaf key in the registry's supported range.
  The first stack/range failures are not successful headed receipts.
  Seed 0, population 256, six-tick bench epoch reached the trait board:
  `RESULT ok`, 26 frames, native hash `99c12f34e2e6eb94`. The frame contains
  the actual terrain and bodies beside the board. `--bench-world --watch`
  resumed its core save with the same tick, hash and standing board (10
  frames, `RESULT ok`). The 3,072-pixel actual-device comparison against the
  old loop's Section matched 137 body, 463 terrain and 2,472 empty picks;
  independent frame tokens stay distinct, and an unpresented revision and
  out-of-bounds coordinates refuse. In two bounded held-save pacing runs,
  dropping five startup callback intervals, Assembly's median/p95 were
  143,324/153,988 microseconds (56 samples), the old loop's
  183,197/210,229 (54). Both preserve the same hash. These are debug host
  callback intervals under concurrent A1/release work, not GPU timings or a
  performance threshold; the old window was 710x590 logical and the new
  window 1000x700, giving both sections 1420x1180 physical pixels.
  Core ownership transfer 1 passed; bench founding/save 1, runtime pins 6
  and save/import 6 passed; four borrowed-picture admission tests pass and
  the head's all-target check passes. A selector click delivered through
  Cambium's leaf picked native body 230 and queued its core input (13
  frames, `RESULT ok`); the earlier run's zero-tick pause assertion was
  invalid because pause arrives after startup work. Its failed receipt is
  retained as `pause-assertion-*`, not counted as acceptance. Receipts:
  `Code/testing/isometry/h6-2026-10-16/`; cap 100 MB, compact test summaries
  and the named headed captures/saves only. **Still open:** ordinary window
  migration, minimap and exact part-inspection integration, followed by
  ordinary watch/input acceptance and A1's admitted default launch. H6 is
  not closed by the bench capture.
- **2026-10-16, workspace hygiene:** the receipt-budget rule now lives in
  `Code/AGENTS.md` as well as the lane queue, so briefs in other repositories
  inherit its 100 MB default. A read-only size inventory found Mere's target
  at 499 GB, with 201 GB of inactive debug incremental data. After checking
  Cargo markers, live owners, absolute paths and the exclusive debug profile
  lock, only its two incremental directories were removed; sources, receipts,
  binaries, libraries and active release output remain. The filesystem
  recovered 182,078,619,648 bytes, reaching 505,574,342,656 free bytes.
  `cache-cleanup.json` in the H6 receipt directory records both paths and the
  free-byte readings. Later builds crossed below the handle-now line again;
  after the same ownership/lock checks and individual MSF-header validation,
  only 414 inactive Mere debug PDB files were removed, recovering another
  51,586,351,104 bytes to reach 538,986,729,472 free bytes. Exact filenames,
  byte counts and guards are in `cache-symbol-cleanup.json`. Source, receipts
  and active outputs remain. The stable Isocosm target remains for A1 and H6/F work;
  no isolated target, Cargo home or worktree was created.

- **2026-10-16, later cache cleanup audit:** after build growth crossed the
  handle-now line, 617 older inactive Mere build-profile PDBs were removed.
  Their bounded paths, Cargo markers, MSF headers and lack of live owners
  were checked. The attempted `.cargo-lock` check failed and PowerShell
  continued, so an exclusive lock was **not established before deletion**.
  `cache-build-symbol-cleanup-audit.json` corrects the preserved original
  receipt's lock claim. The actual `.cargo-build-lock` was found and checked
  exclusively afterwards; this does not retroactively satisfy that guard.
  The filesystem recovered 25,726,193,664 bytes, reaching 540,930,355,200
  free bytes (503.78 GiB). No source or evidence was deleted.

- **2026-10-16, H5 landed:** the core's
  `save::ImportSave` lets a product validate its old grammar and lower it to
  `WorldSave`, with `World::import` replaying native authority. Mesocosm's
  adapter lives in `runtime/world_save.rs`, without a product dependency in
  the core. The head's trace writer now uses the core's atomic file
  replacement and `WorldSave`; watch and replay accept that envelope, GameSave
  and native-only traces. Runtime resume restores the saved controller,
  physical inputs, question, queue and fault, keeping unknown sections.
  The grammar discriminator refuses malformed envelopes without falling back.
  Identical `put_section` bytes now leave the revision alone. The three
  must-use flow warnings were informational matter-move results, with errors
  already propagated by `?`; their callers now explicitly retain and discard
  those transient readings, with their purpose documented. No A1 draw or
  threshold is changed. Receipt budget: 100 MB, only test summaries in this
  plan; the writer test's temporary saves remove themselves.

- **2026-10-16, H5 checks:** core tests with `--features endpoint`: 52
  passed, five existing filesystem tests ignored; runtime `--lib --features
  endpoint`: 15 passed; `runtime_pin`, `one_door`, `ecology_offer` and
  `live_input`: 18 passed. The head's `tests/world_save.rs` drives its actual
  atomic writer, replay and watch reader through a held birth question and
  continued play: one passed. The runtime save tests also cover legacy
  import, unchanged native hash, identical play after resume, unknown binary
  sections, pending input, stored faults, invalid native witnesses and
  malformed formats. Final combined run on the shared head/runtime graph:
  the six runtime save tests and the one head writer/watch test passed,
  including unrelated-section publication with pending input. No window was
  launched; H6's captured bench play, pacing/picking comparison, epoch and
  board remain open. This pass does not remeasure A1 or the F1 to F8 suites.
  The stable `C:\t\cargo-targets\isometry` cache is retained for this
  repository's H6 builds; no isolated target, Cargo home or worktree was
  created.

- **2026-10-16, H4 landed (954 to 957):** the core's driver takes live
  input (`host/src/drive/input.rs`): a plain `Press` and `Pick` the window
  lowers, `press` and `point` hooks on `Product` that make envelopes,
  `current` refusing a pick stamped with a world revision the world has
  moved past, and `offer`, which refuses live input once the backlog reaches
  its cap (default 10) while `queue` stays uncapped for replays and scripts.
  Mesocosm's keymap moved to `mesocosm-runtime` (`runtime/keymap.rs`): E and
  Space nudge toward the focused thing or the played site, S speciates,
  Enter and T answer a question, R commits the board's selected takeable
  offer, X, F, K and G are dev intents when dev keys are live, and a primary
  click nudges toward the pick. `mesocosm-genet` lowers winit's `Key` once
  (`input::lower`) and keeps the host-state keys; its clicks are stamped
  with the revision of the frame they were drawn from, a right-click opens
  the hint menu only on a current pick, hints go through `offer`, and the
  demo script queues the keymap's intents directly. Gates: core 51 passed;
  Mesocosm 81 passed and main's 3 (F1) failed; root `--all-features` and
  Eponym check clean; four headless input tests (`tests/live_input.rs`)
  drive E, Space, S, Enter, R, the dev keys, clicks, a stale pick and the
  cap, and planted staleness and backlog faults fail them at the core and
  in Mesocosm. Not rerun: the `--bench` scenarios, which drive the cambium
  bench over a native session and touch none of this code. Not done: a
  headed key run, since the played head boots only on a generated
  candidate with its 875 witness (H6); at the six-tick test world's board no
  offer is takeable, so R's commit of a takeable offer is exercised only
  where one exists. The continuation panel's answers (W, N) stay on
  `submit_loop_input` (955, reading). `World::put_section` moves the
  revision even for identical bytes, so every published round stales a
  pick; the window stamps after the frame's advance, so it holds.

- **2026-10-15, H3 step 3 (on the lane), H3 landed:** Mesocosm's
  `Runtime` now owns a `Driver<Mesocosm>` over the core's `World`; the
  session lives in the world and the game in its `mesocosm` section, the
  door the endpoint uses (925). `Mesocosm` is the `Product`: `apply` is the
  translation seam, `held` and `answers` the checkpoint, `round` the saved
  loop's round. `Interim`'s settings live in `runtime::Loop`, which
  `interim()` returns, so field reads compile unchanged; every founding path
  still goes through `Interim` and splits it. Each native change is one
  core transaction (`commit` carries a product refusal through unchanged).
  After every call that changes it the runtime publishes the sealed game,
  the driver's queue and fault included, to the section, and before each it
  adopts a game another door published since (the world's revision says
  so). The core's `World` gained `session()` (a read borrow), `section` and
  `put_section`, and the driver `set_max_steps`. *Readings, not ruled:*
  `session()`, `sim()` and `played()` now lend borrow guards, so
  `mesocosm-genet` passes `&` at 17 call sites and drops its frame's guard
  before mutating; the in-memory loop seals an empty queue between
  publishes, since the driver holds the queue; a refused round now leaves
  the session and game as they were, where the old driver could leave them
  part-way. Gates: the six pins pass unchanged, the two traces replaying to
  their recorded hashes through the core's driver; two one-door tests
  (`tests/one_door.rs`): the runtime and the endpoint carry on from each
  other's rounds, the runtime's round stales the endpoint's snapshot (910),
  the save replays both doors' rounds, and an answer through the endpoint
  releases the runtime's hold. Six planted faults caught, the translation
  seam's among them (`Code/testing/isometry/h3-step3-2026-10-15/`); the
  first run missed "never adopts another door's game", which the
  endpoint-answer test was added to catch. `mesocosm-runtime` 36 passed;
  Mesocosm's workspace 73 passed with only the 7 failures 931's triage
  holds; the core 41 and 48; Eponym's `eponym-play` and the root's
  `isocosm-vtt` check clean. The g4 receipt was not rerun: the endpoints'
  code is unchanged and their tests pass. Not measured: every runtime
  change now clones the session for its transaction, where the old driver
  mutated in place.
- **2026-10-15, H3 step 2 (on the lane):** the core drives
  (`host/src/drive.rs`). `Product` names the product's envelope and
  refusal types and supplies `apply`, `held`, `answers` and `round` (926);
  `Driver<P>` holds the world, the queue, the trace, the step ceiling
  (default eight) and the fault, and paces by `Pacing::Clock`, the core's
  `Clock`, or `Pacing::Explicit`, where rounds run only through `step`
  (927). Its step keeps today's `Runtime` semantics, so step 3 can
  re-express it: queued envelopes apply in order before the round, an
  envelope that opens a question ends the step before its round, a held
  advance drains the answers at the front of the queue and banks no time,
  and a refused round stops the run until a restored fault clears it.
  `Clock` moved from `mesocosm-runtime` with its six tests
  (`host/src/drive/clock.rs`); `mesocosm-runtime` now depends on
  `isocosm-host` outright and re-exports it, with society still off in
  Mesocosm's graph. Five driver tests over a toy product on the fixture
  world; six planted faults caught
  (`Code/testing/isometry/h3-driver-2026-10-15/`). The core runs 48 tests
  with every feature and 41 without; `mesocosm-runtime` 31, its pins
  unchanged; Mesocosm's workspace, Eponym's `eponym-play` and the root's
  `isocosm-vtt` check clean on all targets.
- **2026-10-15, H3 step 1 (on the lane):** today's `Runtime` is pinned
  by `mesocosm-runtime/tests/runtime_pin.rs` (six tests), on two worlds
  from the native ecology fixture's generated candidate: `offered`, through
  the 879 offer gate, and `held`, the same candidate with a six-tick epoch
  founded through `Interim` and resumed, where a birth question opens in
  the first round. Pinned: queued envelopes apply first in, first out before
  the round, each with its outcome in the trace (nudged, not played,
  attention, another tick refused, unasked); a cap of three turns a second
  at 30 Hz into ten calls of three, deferring and never dropping, and the
  default cap is eight; a held run takes no round and banks no time; an
  answer drains in a held call, an answer queued behind another envelope
  waits with it, and an answer of another kind does not drain; save and
  resume keep the question, the queue, the tick and the hash, and the trace
  starts empty. Two envelope traces are recorded in
  `tests/fixtures/runtime_trace_{offered,held}.json` (16 and 9 actions,
  the offered one ending at tick 72 with 76 entries) and replay to their
  recorded hashes; rerecord with `MESOCOSM_RECORD_TRACE=1` only when the
  sim's behaviour is meant to change, as the A1 lane's merge may. Six
  planted faults caught, the translation seam's among them
  (`Code/testing/isometry/h3-pin-2026-10-15/`). `mesocosm-runtime` now runs
  37 tests. Found while pinning, and not pinned: a dev Kill opens a loss
  question that no answer satisfies ("saved lost member was not alive at
  round start"), and a save taken there fails to resume with the same
  message; a forced birth the sim refuses still reports success, carrying
  the refused event; sealing the saved loop digests the whole session log
  several times a round, so stepping slows as the log grows (3,000 rounds
  of the offered world ran for minutes in a debug build).
- **2026-10-15, H7 added (923, 924):** `isonetry` is remade as the
  family's session layer; the phase starts with its assessment.
- **2026-10-15, 910 built:** `World` counts the changes it accepts,
  game sections included (`World::revision`), since a change that rewrites
  only a section (the VTT's roster) leaves the cut alone. `Cut` stays the
  native position, which saves and resumes compare. The shared
  `Disclosure` holds the world and catches up at each snapshot and intent:
  if the world moved, through this endpoint or another, the revision
  advances, so an intent made against the older snapshot is stale and the
  next snapshot is a new revision. All four endpoints construct it with the
  world. Two tests, one in the core (two endpoints over one world, a step
  through one and a section-only change) and one in the VTT (a DM's roster
  change stales the player's open table). Four planted faults caught
  (`Code/testing/isometry/r910-staleness-2026-10-15/`); a first fault
  against an explicit moved-world check was equivalent, since catching up
  already advances the revision the ordinary check compares, so that
  check went. Core 37 passed with every feature and 30 without; VTT 7,
  Mesocosm 4, Eponym 3; the core's and the root workspace's all-target,
  all-feature checks are clean.
- **2026-10-15, rulings 909, 910 and 912:** main is pushed with H2 and the
  owned-stock-tests lane (909). Under 910 every endpoint's snapshot records
  the shared world's position and an intent made against an older position
  is stale, so a change through any endpoint stales the others; it is built
  in `endpoint/disclosure.rs`, which all four endpoints share. The g4
  build's output stays in mere's target directory for later receipts (912).
- **2026-10-15, H2 landed (on the lane):** the done-conditions hold. Each
  endpoint's projections, cards and actions pass `LocalCarrier` tests (core
  36 library tests with every feature, Mesocosm 4, Eponym 3, VTT 6).
  `isocosm-host` names no product crate. The four failing shell tests are
  carried or recorded (step 3's entry below). Graphshell's `g4_sessions`
  (mere `0aade866`; `chirograph`, `graphshell-endpoint` and `sessions.rs`
  are unchanged from the pinned `0b9b7a9b`) mounted 9 sessions from the
  four endpoint programs and rendered them: the core's World, Mesocosm's
  played turn, Eponym's next life, and the VTT's six projections. g4
  invokes each distinct advertised intent once, with an empty payload, at
  the mount's revision. Each endpoint accepted its first intent (step,
  roster configuration), later intents in a projection came back stale,
  and intents needing a payload were refused for it (Mesocosm's configure,
  the VTT's previews, apply and assert). Receipt, programs and their hashes:
  `Code/testing/isometry/h2-vtt-2026-10-15/`. The receipt's static page
  overlaps cards, since it places fixed-size cards at positions normalised
  to the scene's bounds. While checking that, the shared disclosure code's
  card footprint went from a radius-6 circle to the 260 by 136 rectangle
  Graphshell's own card endpoints use. Step 4 waits on the capabilities
  being lifted from the reference host (899).
- **2026-10-15, H2 step 2 (on the lane):** every shell panel now lives in
  a product endpoint. Mesocosm's played turn (`mesocosm-runtime`, module
  `endpoint`, feature `endpoint`): a turn card and an answer card per
  checkpoint question read through `continuation::read_loop`; configure,
  step and answer; four tests, stepping included, which carries the two
  shell tests that refused at `configure the explicit played loop before
  stepping`. Eponym's next life (`eponym-play`, the same shape): an Eponym
  card, a card per visible body and, once the taken body has died, a card
  per eligible next life; step keeps a configured Eponym clock; three
  tests. A successful take is untested, since the core fixture holds no
  Eponym mode state. The VTT's six panels (`crates/isocosm-vtt`, 902): one
  endpoint, six projections over one table seat. Table time and party, with
  the shell's seat-cut reading (characters and authored sites); joint
  journeys; the disclosed overmap, display cells as cards, routes between
  anchored cells as relations, and examination through the window's
  attention inside the seat cut; held map knowledge; adventures and goals,
  with polity readings; native operation authoring, six preview kinds,
  apply, and the shell's DM contract assertions. Card actions bind to acts;
  world changes go through `submit_game_access` with the game bytes in the
  world's `vtt` section, and an accepted act stales every projection. Six
  tests on a table world with one authored party drive both seats: the
  roster, a downtime proposal and a player's consent, party choice, joining
  and withdrawing a body, refusal of malformed operations, and a
  `LocalCarrier` pass. Five planted faults were caught
  (`Code/testing/isometry/h2-vtt-2026-10-15/`). Paths this world cannot
  reach are untested: journey approval, packs, storylets, goals, held
  statements, overmap cells and examination, all of which need authored
  content. The root workspace's all-target, all-feature check is clean.
  *Reading, not ruled:* the done-condition's "bench world" is read as the
  core fixture and the table world, small generated worlds founded without
  a witness. The shell kept each window's chosen party and overmap
  settings in its desktop preferences; that store retired with the shell,
  and the VTT endpoint holds those choices as in-memory curation. Each
  endpoint also gained a no-argument stdio example
  (`core_endpoint`, `vtt_endpoint`, `mesocosm_endpoint`,
  `eponym_endpoint`), because `g4_sessions` spawns endpoint programs
  without arguments.
- **2026-10-15, H2 steps 1 and 3 (on the lane):** step 1 serves the core
  as a Graphshell endpoint (`host/src/endpoint.rs`, `endpoint/cards.rs`)
  behind the `endpoint` feature: a world card, site cards with route
  relations, the examined site's group cards with residence relations,
  parts on an inspected group; examine and inspect (curation), step (domain
  truth), save (external effect); stale and undisclosed intents refused.
  Six tests, one through `LocalCarrier`; two planted faults caught after
  the refusal test gained step and save on a site card, which only the
  advertisement stops. A stdio `Discover` against the binary answers the
  World projection. Step 3 (904) retired the desktop shell: `ui/`,
  `desktop`, `products`, `viewport`, `cli`, `mode_sections` and
  `shell_tests` are gone, with the `desktop`, `mesocosm`, `vtt` and
  `eponym` features, their eleven product and window dependencies, and 24
  patch rows Cargo reported unused. `isocosm-host` now names no product
  crate anywhere. Library 30 passed without features and 36 with them;
  `--validate-save` gives the same hash on the H0 gate save. The four
  shell tests that failed on main went with the shell: the two stepping
  through product modes (`configure the explicit played loop before
  stepping`) are re-covered by the product endpoints' tests in step 2, and
  the two viewport-registration tests belong to the heads' `Assembly`
  producers (H6), recorded here as dropped from H2.
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
