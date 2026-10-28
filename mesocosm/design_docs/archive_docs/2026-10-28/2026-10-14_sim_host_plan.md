# The sim host: one embeddable core under three uniform product hosts

**Date:** 2026-10-14

**Status, 2026-10-18:** in progress. Done: H0 to H5 (H2's step 4 waits on case
899), and H6's functional done-conditions, including a captured ordinary
clock-seeded launch; A1 and the generated physical joint-witness gate pass,
and ordinary Mesocosm new worlds select A1 (case 961). H7's local native UI
and exact durable restart pass. The core tick lane (cases 975, 976) cut 15
owner steps on the seed-9 save from 20.6 s to under 7 s, so case 976's 8 s bar
holds. Open: H7's production stack admission and peer setup, case 966's 1.5 s
bar (native scheduling is the floor) and active-update latency. Next: the
scheduling profile; the head's tempo is 2 ticks per second (case 978).

## Why

**2026-10-20, rounds and forecasts, implemented.** Profile the same unchosen
world's ordinary rounds and natural epoch boundary, reuse authenticated forecast
sources and repeated physical searches where measurements justify it. Done when
before/after timings, native histories, scores, product sections, replay controls
and the rebuilt playable agree. Receipts: `Code/testing/mesocosm/rounds-forecasts-2026-10-20/`,
100 MB total cap. Reuse the existing playable target and temporary marked Cargo
home; preserve the separate simulation-execution plan's concurrent work.

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

- **2026-10-20, ordinary round and forecast profile, verified:** the same
  unchosen world reaches the same tick-60 review, with byte-identical native
  history, mode sections and scores, hash `f5767ede7effce0f`. It offers only
  the status quo. `world:weather`
  changes every three ticks and invalidated volumes despite not being read by
  the terrain skeleton. Derived geometry now keys the skeleton's actual
  conditions; full native admission revisions remain unchanged. Arrival searches,
  walking validators, replay and the physical scene share bounded immutable
  volumes. Independent candidates reuse one replay-authenticated source per
  unchanged native cut.
  The crowded tick-56 tail repeats native preparation 164 times, with 140
  birth questions; 133 immediately refuse an absent physical parent. An
  optional native fallback now batches only that refusal prefix before the
  next ordinary birth question. It returns concrete receipts on pending
  attempts, preserves callback order, and publishes through the existing
  explicit spatial command. Newly bound parents still use physical providers.
  Normal release runs reduce that crowded round from 34.13 s to 10.06 s and
  preparation calls from 164 to 31. The resumed tick-59 boundary falls from
  110.14 s to 82.63 s; continuous play reaches its review in 54.98 s. Across
  ticks 46 to 59, total round time falls from 94.89 s to 81.24 s, but median
  time rises from 3.75 s to 5.34 s. These are shared-machine wall samples,
  not a controlled hardware benchmark or a frame-budget receipt. Walking
  preparation and physical forecasts remain expensive: the resumed boundary's
  seven clocks spend 35.22 s in preparation and 14.12 s in step answers; its
  one source replay costs 15.08 s and six forecast clocks cost 48.94 s.
  Controls pass 337 tests, 24 ignored, using unoptimized subject crates over
  release dependencies; the saved-world timings use the normal release build.
  Both continuous and resumed production runs pass save/reload equality.
  The rebuilt head passes 16 frames at tick 60 with 35 visible physical bodies,
  zero projection failures and unchanged native and mode sections. All five
  legacy saves replay to their recorded hashes, including tick 64 and the
  coarse, root-only, whole-body and foraging formats.
  Receipts: `Code/testing/mesocosm/rounds-forecasts-2026-10-20/` (`summary.json`,
  `comparison.json`, `final.png`), 33.2 MB retained of the 100 MB cap.

- **2026-10-17, tempo (978):** with a tick at about 0.45 s after the core
  tick lane (975, 976) and native scheduling alone at 0.3 s, the configured
  10 ticks per second made the clock defer without limit, every active
  callback running the full 8-step ceiling. The head's default is now 2
  ticks per second; TD2's 10 stays the aspiration until scheduling drops.
  The 1,440-tick day of the light field (973) is 12 minutes of play at
  this tempo.

- **2026-10-17, core tick lane landed on `core-tick-966` (966, 975, 976):** cohorts are shared behind an `Arc` and copied on write (`Cohort::entity_mut`), and the pass frame and stage bases hold the pass-start body by `Arc` instead of cloning it. Serialisation and hashes are unchanged. On the seed-9 save, 15 owner steps fall from 20.6 s to 6.7 to 6.9 s (four runs), tick 85 hash `907019ed7b2eca3e` and all ten signature fields equal to baseline; Session clones and drops fall from 13.25 s to 0.55 s, the three physical refreshes per tick from 5.83 s to 0.26 s, so they are not cut further; native scheduling is 4.63 s, now 70% of the cost and the floor. The shipping head (`--seed 1792241946691427000 --frames 3 --no-trace`) reaches tick 75, hash `90c45b8673d88264`, the capture is byte-identical, and its active callbacks sum to 6.65 s against 33.6 s before (revision 22 against 20: callback grouping differs after the speedup, as before). 976's under 8 s holds; 966's 1.5 s does not and stays open for a later ruling with the scheduling profile (`produce-3`, `move:0-1`, `upkeep-3`). Controls: `isocosm` 636 passed, 18 ignored, with the 14 process, 4 port-admission and 3 target controls; `mesocosm-runtime` lib 23, `runtime_pin` 6/6 unrerecorded, `ecology_offer` 9, `world_save` 6; `isocosm-host` 58 with 5 ignored; the Mesocosm all-targets and root all-features all-targets checks are clean. Receipts: `Code/testing/isometry/core-tick-2026-10-17/` (`core-tick-report.md`), 1.9 MB of 100 MB.

- **2026-10-17, core tick split (966, step 1):** on the seed-9 save at `c58262a5` plus profiling spans, 15 owner steps (tick 70 to 85, hash `907019ed7b2eca3e`) take 20.6 s unprofiled and 26.3 s with spans. Session and Game copying were timed apart from the refresh checks, as the plan asked. Session clones and their drops are 13.25 s (50%): seven clones per tick (round, clock, spatial prepare, spatial command, and three physical refreshes), each about 62 ms to clone and 36 ms to drop, of which `state.population` (2,000+ cohorts, 28.5 MB serialised) is 95%. Native scheduling is 7.75 s (29%), spiky (`move:0-1`, `ecology:produce-3`, `ecology:upkeep-3` at 0.2 to 0.6 s on their due ticks). The refresh member checks cost nothing measurable (0.008 s) and the Game clone is 1.5 ms. The whole of `physical.refresh` (5.83 s) is its own Session clone and drop. One refresh per tick therefore saves about 3.9 s; the budget of 1.5 s for 15 ticks (0.1 s per tick) is below the cost of a single Session clone plus drop. The lane stopped here to put the fork to Mark before any optimisation. The profile log cap is now `ISOCOSM_PROFILE_LIMIT` / `MESOCOSM_PROFILE_LIMIT` (it had truncated earlier phase sums to the first ticks) and `MESOCOSM_ACTIVE_STEPS=N` runs N owner steps in `mesocosm-active-profile`. Receipts: `Code/testing/isometry/core-tick-2026-10-17/` (`split-report.md`).
  Mark ruled 975 (share cohorts behind an Arc, copy on write, hashes and serialisation unchanged) and 976 (done when the seed-9 save runs 15 ticks in under 8 s, replacing 1.5 s; the tempo question stays open). The refresh cut is no longer a step of its own: its checks measure as free, and it is re-examined only if the measurement after copying still points at it.

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


---

*Archived 2026-10-28.* Cut, with the other active plans, into a new set: what was
still open, deferred or a follow-on here now lives in `2026-10-28_sim_host_plan.md` (in the product's
`design_docs/`), which cites this file by line. This copy is history: its
status lines, receipts and claims are as they stood, not current.
