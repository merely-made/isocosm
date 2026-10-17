# The sim host: one embeddable core under three uniform product hosts

**Date:** 2026-10-14

**Status, 2026-10-17:** in progress. Opened by rulings 884, 892 and 895;
carries 887's first milestone; forks settled by 896 to 898. H0, H1, H2 and H3
landed (H2's forks ruled 899 to 904); H2's step 4 waits on the lifted
reference-host capabilities (899). 2026-10-16: H4 (954 to 957) and H5
landed. The ordinary Mesocosm window now runs on Assembly, renders through
the section producer and resumes core saves. Its native controls and replay
pass. A1 and the generated physical joint-witness gate pass. Under 961,
Mesocosm's ordinary new-world source now selects A1, retaining shared Stock
defaults. H6's functional done-conditions now pass, including a captured
ordinary clock-seeded launch. Active native updates remain slow. H7's local
native UI and exact durable application restart pass; production stack
admission and peer setup remain open. Audit 2026-10-17 (964 to 967): the
H3 pins failed under 958's movement ties and are rerecorded; the next
responsiveness lane aims at the core's tick cost (966).

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
the accepted log. The family stack owns authenticated peer identity and
admitted connections. Each game maps an authenticated peer to its native
participant and seat. Projection/seat access does not grant full-World
replication; that authorization remains a separate gate. This is the route
for the VTT and later the other games, not evidence that all three have shipped
peer play. It starts with an assessment, whose forks go to Mark before code:
what Graphshell's network carrier (`graphshell_endpoint::network`)
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

The assessment brought these forks before H7 code (923). Mark chose the
recommended path for both on 2026-10-17 (959, 960):

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
revision. These are the first implementation gates for the chosen paths.
The assessment itself claims no H7 code or network receipt. Implementation
starts with the core's atomic replication boundary and serialized peer records.

**Later.** Eponym's `session` on the core (founding a generated world, asks
through the UI, succession in the client), then the VTT over the sim
(in-app campaign founding from a seeded draw). Each gets phases here when it
starts.

## Progress

- **2026-10-17, audit (964 to 967):** a read of the tree at `a7d02a7f` with
  the compile gates rerun: the Mesocosm all-targets check and the root
  all-features all-targets check pass, the whole tree is under the line
  ceiling, and the Mesocosm workspace tests pass 113 with one failure, seven
  explicit ignores and the release-only physical joint witness skipped (its
  receipt is `generated-physical-witness-gate.json`, 1,182.93 s). The failure
  is `runtime_pin`'s recorded traces: the offered trace replays its 16
  actions and 5 outcomes to tick 72 and 76 entries, to a different state
  hash. Bisected in place under 964: the suite passes at `c0c54a88` and
  fails at `89db70a5` with HEAD's hash, so 958's seeded movement ties are
  the cause and the later active-update commits changed neither world. Both
  traces are rerecorded at `a7d02a7f` (offered `13219767039782569982` to
  `4535962164617645755`, held `1305873899039827589` to
  `11223436107781315567`) and the suite passes 6/6. The H5 receipt of
  2026-10-16 was this suite's last run; no lane that moved the sim after it
  reran the pins. Findings carried: the body tenant lights with ambient only
  (`LightBlock { ambient, ..Default::default() }` leaves the sun `None` and
  shadows off), so the seed-9 capture lights its bodies only by the per-face tint isometer bakes in (face_shade), with no sun; the default
  launch's first capture is a loss checkpoint with a withdrawn viewport, and
  no capture with a visible site exists after 08:52, the five inspected PNGs
  since being byte-identical copies of that frame; the configured tempo is
  10 ticks per second against about 1.9 s per tick measured (15 ticks in
  28.7 s of callbacks), the time inside the core. Under 966 the next
  responsiveness lane aims there: time Session/Game copying and refresh
  checks first, one physical refresh per tick, done when the seed-9 save
  runs 15 ticks under 1.5 s on the shipping head, the host unchanged.
  Mesocosm's manifest still patched vello with nothing depending on it; the
  patch goes under 967 with the merge of origin's pack-authoring tool and the
  1.99.1 toolchain (965). Merge gates: the root all-features all-targets
  check (3m09s), the Mesocosm all-targets check (2m45s), the pin suite 6/6
  and `isometry-system`'s library (49) and `pack_author` integration (2)
  tests pass; the `pack-author` bin's
  `nested_presets_real_validation_and_no_implicit_runtime_change` fails on
  this Windows checkout because autocrlf converts its two TOML fixtures to
  CRLF and its needle carries a bare newline; it passes on LF copies and
  fails the same on plain origin/main. Pushed with that recorded (968); the
  fixtures are pinned LF by a narrow attributes rule in the following commit
  (969). The merged lock adds exactly origin's pack-authoring closure at
  origin's versions.

- **2026-10-17, active-update performance lane:** profile the 32 to 50 second
  active callbacks, remove redundant work while preserving native commands,
  hashes, product section bytes, revision/refusal behavior and replay gates.
  Required receipts are bounded before/after phase timings on the same qualified
  A1 seed-9 save, matching native/product signatures, focused regression controls,
  compiler checks and a final native head capture/profile. Owner:
  `Code/testing/isometry/active-updates-2026-10-17/`, total cap 100 MB. Reuse the
  existing save and stable target. One Cargo owner; the initial passes used one
  compiler job. Subsequent native and head builds use two jobs while free RAM exceeds
  10 GiB, reducing the independent library/test compilation wait without parallel
  test or profiling workloads.
  First measurements isolate redundant historical answer replay (23.552 s),
  repeated native witness hashing and full loop readings used only as pane
  predicates. A process-local receipt, minted only by the validated live round
  or mandatory full restore, replaces historical reconstruction for the exact
  same continuation. Changed continuations fall back to replay; current access,
  native/product/question cut and refusal checks remain mandatory. World cuts
  cache only accepted native authority, invalidated at all three native publish
  paths. The phase hint is invalid after external publication or failed publish.
  The same saved answer is 1.963 s and the following tick 1.670 s (formerly
  2.250 s), with exact native save, section, reading, trace, tick and revision.
  Controls: 56 host passes, five explicitly ignored atomic-file tests; 22 runtime
  passes, two explicit profiling ignores; two additional geometry controls pass.
  The first head repeats the prior clock seed explicitly: all 15 native steps
  reach tick 75 and hash `90c45b8673d88264`. Active callbacks are 5.006, 25.962
  and 21.458 s, still too slow. Scene/UI refresh is below 16 ms; the remaining
  time is native clock preparation, including a 7.679 s single-tick spike.
  Clock delivery groups the same steps differently after the speedup, producing
  revision 21 rather than 20 through an additional accepted intermediate product
  publication. This is not a claim of identical callback boundaries. Receipts:
  `saved-update-comparison.json`, `first-head-comparison.json`, the bound profile
  logs and native captures under the owner above. Deeper native pass timing
  identifies repeated prey-ledger construction and two complete source hashes
  per spatial tick. The owner clock now stages and publishes under one exclusive
  Session borrow; the separate Prepared API retains its stale-source guard.
  Prey holdings are cached only within the pass-start Frame, and immutable
  admitted process definitions retain their content addresses. Three atomic
  spatial controls and three target controls pass, including pending/refused
  unchanged sources, exact guarded/atomic saves, and changed/split/dead/born
  targets in both execution modes. Before the address cache, the seed-9 answer
  is 1.896 s and its next tick 1.225 s, with every recorded signature exact
  (`atomic-books-comparison.json`). The address lookup probe compares identical
  ordered matches: 100,000 resolutions take 109,798 us freshly hashed versus
  252 us with retained addresses. This microprobe does not predict head latency.
  All 14 process controls, four port-admission controls and three target controls
  pass against the final address-cache build. The release native test harness
  takes 3m18s to compile, while those controls run in 0.03 s total.
  A small `mesocosm-active-profile` executable reuses the shipping runtime's
  dependency graph and the existing qualified save, avoiding a library test
  harness rebuild for repeated measurements. Set `MESOCOSM_ACTIVE_SAVE` to
  that save and `MESOCOSM_PROFILE_ACTIVE=1` for the bounded phase log;
  `ISOCOSM_PROFILE_ACTIVE=1` adds native pass timing. The address-cache head
  preserves tick 75/hash `90c45b8673d88264`, with active callbacks 4.817,
  23.468 and 16.368 s and a 52.366 s full witness. The final saved answer
  is 2.101 s, next tick 1.301 s and restore 57.012 s, with every signature
  unchanged. These are still long callbacks. Its 15 active ticks spend
  10.926 s in native scheduling, 8.314 s in three physical refreshes per tick,
  4.391 s in immediate owner-cut hashing and repeated physical seals/validation.
  The saved game's 8.1 MB ecology witness is unchanged admission evidence.
  A subsequent pass shares that witness with copy-on-write mutation and caches
  its exact encoded bytes for physical digests; the normal saved encoding and
  all replay/validation gates remain intact. Immediate owner rounds use an
  exclusive synchronous World transaction; delayed public Prepared/product
  calls retain their original source-cut checks. A single exact-key geometry
  cache now survives owner rounds; public wrappers retain fresh local caches.
  The key covers all native volume inputs, site/window and placement rules;
  body and visit checks run on every refresh. The final head builds geometry
  five times across its 15 active ticks (142 ms total).
  Final active callbacks are **2.817, 15.190 and 10.674 s**, a **64.95%**
  reduction in total callback time from the original 32.192/49.634 s run.
  The same 15 native steps reach tick 75/hash `90c45b8673d88264`; the final
  capture shows the same legitimate loss checkpoint and 352 living descendants.
  Revision 21 has the previously explained extra accepted intermediate product
  publication. The final saved case answers in 1.668 s and steps in 0.962 s,
  preserving every native/product/reading/trace/revision signature. Restore is
  57.379 s and full default witness 51.428 s. All 58 host controls pass (five
  explicit atomic-file ignores); the final real-witness controls prove exact
  serialization/digest, COW isolation, cache invalidation and unknown-field
  refusal. Warm physical digest is 7.2 ms. The rebuilt shipping head passes.
  Receipts: `retained-geometry-binding.json`, `retained-geometry-head-comparison.json`,
  `retained-geometry-saved-comparison.json`, phase logs and inspected PNG,
  `current-world-controls.log` and `final-seal-controls.log` under the owner above.
  Remaining active latency is substantial: native scheduling takes 9.948 s,
  clock preparation 14.187 s and physical refresh 5.995 s across those 15 ticks
  (nested totals overlap). Geometry construction accounts for little of that
  refresh time; Session/Game copying and refresh checks need separate timings
  before another optimization. This pass does not qualify an interactive frame
  budget. The retained cache is one volume/Places entry, not saved authority.

- **2026-10-17, H7 owner-thread bridge and lifecycle controls:** the shared
  family layer now provides a bounded admitted-stream request bridge with an
  owner wake callback, explicit authenticated-peer keys, response deadlines
  and cancellation checks. Its record queue reserves capacity before draining
  the host, preserving accepted records on full or closed queues. Host and
  replica lifecycle wrappers compose the supplied transport, protected key,
  store, signed checkpoint and JoinedSpace; publication retry checks persisted
  records and checkpoint signing requires the exact signed-log tail. All 34
  library controls pass in `admitted-bridge-lifecycle-queue-controls.log`,
  bound by `owner-thread-lifecycle-qualification.json`, including a real
  two-peer lifecycle, section publication, exact replica
  convergence, retry, stale-checkpoint refusal and clean leave. Identity and
  admission remain caller supplied under 962. Full-World authorization is
  separate from projection access. The standalone product admission provider,
  shipping peer setup and full H7 acceptance remain open.

- **2026-10-17, H7 headed native action and restart:** the shipping Isometry
  Genet UI accepts a native VTT action through its family host, records one
  accepted native suffix entry, and keeps the opaque VTT section unchanged.
  The file ledger is an exact replica. Reopening the same campaign resumes
  durable mode at sequence 2, revision 2 and native hash
  `b05e9d08aaf0caadf7159b2d85d89c31e0b28734edd35fd07b9f9cfe301d1d91`.
  Both headed captures were inspected. The run used the exact executable
  bound by `profiled-native-action-binary-binding.json`; app-only dev opt-level
  was 0 for functional UI responsiveness, so it makes no performance claim.
  `shipping-native-ui-restart-qualification.json` and its logs/captures are
  under `Code/testing/isometry/h7-2026-10-17/`. This qualifies the local VTT
  action and application restart path. Production peer setup and standalone
  stack admission remain open; full-World authorization is separate, and
  power-loss durability is excluded.

- **2026-10-17, H6 functional done-conditions complete:** the rebuilt ordinary
  default selects clock seed `1792241946691427000` on its first structural
  candidate and passes the full native witness in 85.731 seconds. The head
  runs five frames, records two viewport renders, takes 15 native
  steps and saves one inspected PNG without a fault or dev assistance.
  Tick 75 has native hash `90c45b8673d88264`, World revision 20 and a legitimate
  loss checkpoint with 352 living descendants. The captured terrarium is
  withdrawn because this checkpoint has no current site; the UI offers body
  103 after body 102. The earlier explicit seed-9 capture qualifies actual
  terrain and body presentation separately. Held frames now take 23.0 to
  25.4 ms, with two held callback intervals of 23.5 and 24.2 ms. Active callbacks
  still take 32.2 to 49.6 seconds, so active simulation performance remains
  open. The GPU readback deadline is unchanged. The earlier short-epoch bench
  remains the trait-board receipt; this default run reaches a loss checkpoint,
  not its epoch board. The default capture uses `--no-trace`; it claims no new
  save/replay for this clock seed, retaining the earlier exact seed-9 core
  save/replay qualification. All H6 functional conditions are now represented
  by their named receipts, bound in `held-view-default-qualification.json`
  and the earlier manifests. H6's retained receipt owner is 55,969,930 bytes,
  below its 100 MB cap at qualification.

- **2026-10-17, H6 capture-delay diagnosis:** the first bounded ordinary
  clock draw, seed `1792241059404538200`, passes its full witness in 70.911
  seconds and reaches a native birth checkpoint at tick 73, unassisted and
  without a simulation fault. Its final capture fails the unchanged five-second
  native readback deadline. `bounded-default-readback-failure.json` preserves
  that failure. Profiling a separate existing seed-9 save gives valid pixels
  but exposes 6.64 to 6.83 seconds in frame hooks, compared with 1.25 to 1.36
  seconds inside product work. Three `Runner.update` calls rebuild the view,
  including presentation bookkeeping before readback polling. The app now
  defers bookkeeping and minimap transfer; unchanged held World cuts retain
  their semantic view. Native changes, input handlers, pumps and expired
  selections still refresh it. The new view-refresh control passes in 8.07
  seconds, the all-target/all-feature head compiler check passes, and an
  independent review finds no stale UI/pick issue. The capture deadline and
  witness gate are unchanged. `held-readback-diagnosis.json` records the
  evidence; a Probe hash cache was rejected because hashes run after readback.
- **2026-10-17, H6 bounded default and held-frame controls:** under 963 the
  unchosen default tries at most eight structural candidates, configurable
  from 1 to 128. Each seed is disclosed; only the typed missing-consumer/food
  preflight failure permits another draw. Explicit seeds get exactly one
  candidate. Selection runs one full native witness; witness failure remains
  terminal. All six startup controls pass in
  `bounded-candidate-library-final-controls.log` (27.46 seconds). The earlier
  binary-target invocation ran zero tests and is excluded by
  `bounded-final-control-scope-audit.json`. The runtime also skips unchanged
  held frames only when its last successful publication matches the current
  World revision and its intent queue is empty. Three controls pass in
  `held-advance-final-controls.log` (3.53 seconds), covering queued answers,
  discarded held time, external revisions and publication refusal. Failed
  publication never marks a revision clean. These remove redundant work,
  without qualifying interactive performance.
- **2026-10-17, H6 scene-source cache:** successful scene preparation is reused
  only for the same World revision, played member, site and physical/coarse
  branch. The borrowed producer checks its presentation fingerprint before
  cloning the scene. Its invalidation control and all-feature compiler gate
  pass. The optimized ordinary seed-9 watch resumes the exact saved native
  hash, holds tick 70 with zero native steps, and captures the birth checkpoint
  in seven frames without a fault. `--no-trace` avoids writing another full
  save for capture-only work; its conflicting explicit trace control refuses.
  `scene-cache-watch-qualification.json` binds the compact receipts. Held-frame
  work remains 1.06 to 1.35 seconds, with callbacks 5.75 to 8.47 seconds apart;
  this establishes correctness, not interactive performance.

- **2026-10-17, H6 generated seed-9 head and exact replay:** the optimized
  ordinary head, with explicit seed 9 and A1 selected by Mesocosm's default,
  passes its native witness in 49.571 seconds. Five frames and three viewport
  renders reach tick 70 and a native birth checkpoint, unassisted and without
  a fault. The captured terrain, bodies, checkpoint and minimap were inspected.
  The core save replays to the recorded hash `ae57d13f26b17f4f`. Controlled
  member 70 has no saved physical placement, while member 102 does; the existing
  native site view therefore supplies presentation without assigning the
  controlled member another body's pose. The source, executable, save, PNG and
  logs are bound by `release-seed9-head-replay-gate.json` under the H6 owner.
  The generated run is slow: callback intervals range from 1.9 to 27.1 seconds,
  with concurrent release compilation. This is a functional generated-head
  receipt, not real-time performance or a generated epoch-board receipt.
  The first actual unchosen release draw, seed `1792230697480612000`, correctly
  refuses before its witness: its eight cohorts occupy lineages
  `[0, 2, 2, 3, 2, 2, 2, 3]`, with no living consumer. The failure is preserved
  in `release-default-ineligible-draw.json`. Ruling 963 now permits a bounded
  structural candidate search for unchosen seeds; explicit seeds remain exact.
  Explicit seed-9 success does not close the default-launch condition.

- **2026-10-17, H6 fresh-run handoff qualified:** the
  ordinary generated path retains the final Session from its actual witness
  run after one full native witness replay. Non-serializable, privately
  constructed validated/admitted run objects carry that immutable proof into
  role admission and the core World. The fresh product handoff performs
  structural checks; public decoded witness, admission, save and restore
  validators retain full replay. This removes repeated validation and loading
  of the same in-process witness without caching trust in saved bytes.
  The Mesocosm workspace/all-target/all-feature compiler check passes. The
  final optimized offer/restore suite passes all nine controls in 346.62
  seconds, including the full seed-9 physical assessment, offer, save and
  replay in both execution modes, handoff/continuation equality and
  tamper/refusal checks. `validated-run-release-qualification.json` binds the
  current runtime source, executable and log. The explicit seed-9 head and
  saved replay pass above. At this checkpoint the unchosen default remained
  unqualified; the later default-capture receipt above closes that condition.
  Two pre-optimization default trials were stopped without a capture; the
  later trial's seed, source and binary hashes are preserved in
  `cached-head-build-binding.json` and
  `cached-head-repeated-validation-superseded.json`. The development trial
  with clock seed `1792226307408123200` passed native admission in 1,207.771
  seconds, then exposed the unplaced-controller view selection error. It was
  stopped without a captured frame or saved trace;
  `validated-debug-default-view-boundary.json` records that limit. The later
  optimized seed-9 run above exercises the corrected view selection. All are
  under the H6 receipt owner.

- **2026-10-17, H7 bounded restart ledger:** `497644dd` adds an opt-in
  single-writer file ledger containing the initial World checkpoint and all
  accepted records, with a configurable 64 MiB default cap. It persists a
  candidate record before publishing the World, journal or outbox. A failed
  replacement or capacity refusal leaves those authorities unchanged. Resume
  validates and replays the checkpoint and suffix. Twenty-seven generic
  family/network controls pass, including writer exclusion, reopen, atomic
  refusal, deterministic re-signing with the original host key and continuation
  of the signed backlink. The all-workspace, all-targets, all-features check
  passes. Receipts are `durable-ledger-receipt.json` and
  `durable-family-compiler-qualification.json` under the H7 owner below.
  The staged file is flushed and synced before replacement; parent-directory
  sync and power-loss durability are unqualified. The later actual-product
  restart control below qualifies its exercised harness; Genet's shipping VTT
  authority migration and production reconnect lifecycle remain open.

- **2026-10-17, H7 actual VTT restart control:** all five actual product-network
  controls pass (73.23 seconds, zero failures or ignores). A file-backed VTT
  host persists its endpoint-section and native join records, shuts down both
  transports, reopens its ledger with the original signing key, bootstraps a
  fresh peer with the original signed checkpoint over the admitted stream,
  and republishes the complete stored suffix through JoinedSpace. A subsequent
  offered `vtt.leave` produces native history and continues the signed family
  sequence and parent digest. Both replicas converge on the full World, sections,
  revision and native state. Failed test assumptions and the missing-import
  retry remain in the receipt directory. The passing log is
  `durable-product-reconnect-member-controls.log` under the H7 owner.
  This is an actual endpoint/transport control, not a shipping Genet UI or
  power-loss receipt.

- **2026-10-17, H7 shared-World gate and live carrier (959, 960):** a
  core publication gate now journals real endpoint and window transactions
  before the same World publishes. It checks append-only history, unchanged
  genesis/branch, exact native suffix replay, section changes, revision and a
  configurable bounded outbox. Refusal leaves both World and journal alone.
  A host can resume from a verified checkpoint and suffix. The family API can
  be built without the interim VTT layer; the default `vtt` feature preserves
  that API, while `family-net` enables its own stack dependencies.
  Twenty-two generic family/network controls pass, including the real admitted
  Graphshell stream, host-signed JoinedSpace records, opaque section updates,
  verified stored-suffix recovery, signed checkpoint-bootstrap verification,
  and clean carrier/lane shutdown. A negative
  control caught cached signature verification trusting a mutated header;
  the trust boundary now decodes signed bytes and checks the fresh header
  hash before body/record checks. Failed controls remain in the receipt
  directory. Signed operations also require the family parent digest before
  persistence and across recovery cursors. Four actual product-network
  controls pass: Mesocosm configure, Eponym step, VTT join, and a fresh VTT
  peer that fetches a signed checkpoint over the admitted resource stream
  before applying a signed accepted-record suffix. They use the admitted
  stream and host-signed JoinedSpace records, with full native saves, opaque
  sections, accepted sequence and World revision converging. The Eponym
  endpoint persists an absent mode as empty bytes; its step has no native
  command suffix. Each product's local endpoint control also passes.
  The checkpoint includes the full World and every opaque section; its
  invitation descriptor must be issued only to peers authorized for full
  World replication. Projection access alone does not grant that authority.
  `signed-bootstrap-receipt.json` binds the current source and logs. The v1
  `live-carrier-receipt.json` cited by `7dd450be` was overwritten; no exact
  backup was found, while the underlying logs remain. The receipt-directory
  audit is `Code/testing/isometry/h7-2026-10-17/receipt-manifest-audit.md`.
  JoinedSpace shutdown cancels its receive
  drain; already indexed records remain recoverable, without promising that
  every queued delivery completed. At this checkpoint World/log persistence
  had no durable ledger; the later bounded restart slice above adds one.
  At that checkpoint Genet still used the old DM-authority core. The later
  headed native action and application-restart receipt above supersedes that
  UI limitation; production peer setup remains open. Receipt owner
  and cap remain
  `Code/testing/isometry/h7-2026-10-17/`, 100 MB.

- **2026-10-17, A1 complete matrix (958), default choice (961):** played,
  bare and bodied A1 each qualify 20/20; all three Stock controls are 0/20
  with extinction. All 120 actual-round conservation streams and both final
  native replay modes are independently checked. The complete Q3 summary
  binds 95,364,456 raw matrix/recovery bytes, with the full retained scope
  under 100 MB and failed partial receipts preserved. The last thirteen
  Stock receipts use exact canonical replay recipes instead of duplicate
  Genesis bytes; current directing controls pass 52, fail 0, ignore 17.
  The generated physical offer now passes assessment, offer, GameSave and
  exact native replay in Individuals and Grouped (1,182.93 seconds total).
  `generated-physical-witness-gate.json` binds its source, executable and log
  under the H6 receipt owner. Under 961, Mesocosm now selects A1 for ordinary
  new worlds; the shared generator retains Stock. Authored witness inputs
  retain their previous default declaration or their explicit named rates.
  The first actual generated launch was stopped without a window capture or
  saved trace. Its clock-sourced seed was not logged and cannot be recovered.
  The later explicit seed-9 capture and exact saved replay above supersede
  that failed launch without qualifying its missing evidence.
  The physical role declaration comes from living founding populations
  and their actual producer/consumer definitions before any witness Session
  exists; seed 9 has producer 3 and consumer 1, with no producer 0. A
  population-60 fixture has neither role and refuses rather than claiming
  empty evidence. The product spatial driver and `isometer-space` are optimized
  at level 1 in development builds; the unoptimized control was superseded and
  retained. A one-entry transient geometry cache spans each atomic clock,
  keyed by exact site, window, Places rules and native terrain revision.
  Current body, visit, patch and eye checks still run, and the cache is absent
  from saved authority. Two cache controls pass; the explicit seed-9 timing
  probe also passes, with one physical member, two geometry requests and one
  cache build (593.3728 ms uncached, 278.6069 ms cached in the optimized
  geometry profile). This is a geometry measurement, not a startup speed claim.
  Logs are `spatial-clock-cache-optimized-controls.log` and
  `spatial-cache-seed9-optimized-geometry-controls.log` under the H6 owner.

- **2026-10-17, H7 serialized peer slice (959, 960):** `isonetry::family`
  now carries versioned, bounded JSON checkpoints and accepted records over
  the actual core `World`. One exclusively owned host accepts native
  commands and opaque section changes atomically. The initial checkpoint
  binds the record chain; sequence, predecessor digest, native suffix/time,
  native hash, section digest and world revision verify before a replica
  publishes. Unknown bytes survive and deletions are explicit. No-op and
  refused requests consume neither accepted sequence nor revision. Core
  transactions now leave the revision alone when durable authority and
  sections are identical, with overflow refusal before publication.
  Core/endpoint controls: 55 passed, five existing filesystem cases ignored.
  Twelve serialized two-peer controls pass, covering actual native plus
  section updates, time-only changes, failure after an earlier native command,
  bad post-state with a valid digest, ordering, checkpoint resume, substituted
  roots, shared-world refusal, versions and wire limits. The generic family
  and network controls now total 22. Receipts:
  `Code/testing/isometry/h7-2026-10-17/`, cap 100 MB. The admitted-stream /
  signed JoinedSpace integration passes for all three products, including a
  fresh VTT peer's signed checkpoint and suffix. This does not qualify
  crash-atomic World/log persistence, Genet UI authority retirement, or
  product host restart continuity. The GM store and proposal lifecycle remain
  in `isonetry` under 921.

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
