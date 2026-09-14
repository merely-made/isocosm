# Genet document host for Paredros

**Status:** planned, 2026-09-13. Assessment accepted by Mark the same day:
route, plan placement, pin alignment and the GlyphCSS prior-art row were each
ruled explicitly.

**Owns:** how Paredros presents one played session through genet and
netrender, with movement, combat, equipment and inspection all reading the
same `GameState`. This is the Paredros consumer of Mesocosm's
[orthographic voxel presentation plan](../../mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md)
rulings 2, 13, 14 and lane L7. It does not own the shared scene contract,
the appearance crate (L2), ground tile layers (L4), or Livery's CSS work;
those stay with Mesocosm and genet.

**Supersedes:** the presentation half of the functional loops plan's
"rendered-world/body-sheet join" follow-up. Session, movement, combat and
anatomy rules stay in that plan.

## Why not the renderling tenant

The assessment first proposed a renderling tenant inside the timed-action
host. Mark rejected it: the wing presents through genet and netrender, and
renderling exits Paredros once L7 holds. Nothing new is built on renderling.
`room`, `d1_depth`, `crossing` and the residency bins keep building until L7
retires them under Mesocosm's ownership; this plan neither patches nor
archives them.

## What exists

| Piece | Where | State |
| --- | --- | --- |
| Connected session driven by keyboard and mouse | `paredros-client` `timed_action` bin | Text HUD only; movement, strikes, injury, rest, dressing pickup, save/load |
| Subject sheet and equipment panels | `paredros-client::body_sheet` | Private `EquipmentSession` with fixed subject 1 |
| Rendered world | `room` bin over the S0 `Probe` | Renderling; no `GameState` |
| Terrain tracer with orthographic slab camera | `mesocosm-lens` | Shared; Paredros already binds it under `r1-proof` |
| Live body renderer, depth-attached, cached geometry | `mesocosm-render::live_body` | Shared; renderling-free |
| Cambium document host, producer registry, custom leaf | mere `cambium*`, Bench B | Landed 2026-09-13 in Mesocosm's `--bench` |
| Scenario driver | `genet-probe` | Landed; Mesocosm's bench wraps it in a local `probe::Lane` |

Mesocosm's `Section` is not reusable here: its body layer iterates Mesocosm
organisms from a Mesocosm `World`, and Paredros subjects are anatomies in
`GameState`. The tracer and the live body renderer beneath it are.

## Pins

Paredros and Mesocosm already share Mere `4f4de1d05ec99461f7fa3cdc4e514e904a999213`.
Paredros pins netrender `c77b0be8` (two commits behind Mesocosm's `3961aca9`)
and reaches genet only through a `parley` patch at `3a7b5023`, 134 commits
behind Mesocosm's `101d9e9a`. Cargo keys git sources by URL plus reference,
so a second reference is a second crate identity of one family; Bench B's
receipt proves the Mesocosm graph at exactly one revision per family.

Ruled 2026-09-13: Paredros moves to Mesocosm's netrender and genet revisions.
The Isometry root workspace keeps its own older pins; it is a separate
workspace and this plan does not touch it.

**Correction, same day, from the P0 lane:** the first draft said to drop the
`parley` patch. Mesocosm's own manifest still patches `parley`, `taffy` (as
`genet-taffy`) and `ipc-channel` from genet at `101d9e9a`, because `[patch]`
does not inherit through a git dependency and published parley 0.10 lacks
three APIs genet-livery is written against. Paredros's patch was stale, not
wrong in kind. P0 therefore re-points `parley` and adds the `taffy` and
`ipc-channel` rows, matching Mesocosm line for line.

## Lanes

Each lane names its owner and a done condition. P0 blocks the rest. P1 and
P2 share the manifest, so they run in sequence, not concurrently.

### P0. Manifest alignment (Terra)

Add `cambium`, `cambium-rootstock`, `cambium-genet-winit-host`, `genet-probe`,
`paint_list_api` and `paint_list_render` to the Paredros workspace at
Mesocosm's revisions; move `netrender` and `netrender_text` to `3961aca9`;
re-point the `parley` patch and add the `taffy` and `ipc-channel` patches at
genet `101d9e9a` (see the correction under Pins). Renderling, spirv-std and
crabslab stay.

**Done when:** `cargo check --workspace --all-features --all-targets` passes
for the Paredros workspace with one revision per git family in `Cargo.lock`;
the 132 world tests and 6 native handler tests still pass; the timed-action
window smoke still presents.

### P1. Scene producer (Terra)

A `paredros-client` module implementing `cambium_rootstock::TextureProducer`
over the lens tracer and the live body renderer. Inputs are one `GameState`:
its ground for terrain, every living subject's current anatomy and precise
pose for bodies, and the played subject's pose for framing. The camera is
orthographic, per ruling 1. Depth is one attachment shared by terrain and
bodies, per ruling 13. Bodies enter as instances, per ruling 14; no per-body
surface. The producer skips rendering when ground revision, anatomy
revisions, poses and size are unchanged and `needs_frame` is false.

**Done when:** a headless GPU test renders the keeper and the target from the
timed-action fixture over the generated ground and proves, by pixel query,
that the nearer body occludes the farther and that terrain occludes both; a
severed part disappears from the next frame without re-uploading unchanged
geometry; a fractional pose from `AdvanceMotion` moves the drawn body by the
projected fraction; and an unchanged frame returns `None`.

### P2. Document host (Luna)

A new `paredros-client` bin running `cambium_genet_winit_host::run` with a
root view of the viewport leaf, a subject sheet panel, an equipment panel and
a status panel, all built from one `Session`. Keyboard movement and charged
strikes reach `GameState` through the existing intent path used by the
timed-action host; pointer clicks on the viewport resolve to a subject or
part through the producer's query, with DOM controls addressing the same
identities. The timed-action bin stays as the text receipt until this host
covers every action it supports.

**Done when:** the played subject moves, strikes, is injured, rests and picks
up a dressing from this window; the sheet panel shows the severed part and
the attached dressing that combat and equipment produced; save and load
round-trip through the same `GameSave` version; and the bin builds with no
new renderling use.

### P3. Scenario acceptance (Luna, root verifies)

A `genet-probe` scenario under `paredros/testing/session/` that moves the
played subject, charges and releases a strike, severs a part, and captures
before and after. Receipts and captures follow Mesocosm's
`testing/bench/receipts/<date>/` layout.

**Done when:** the scenario asserts a changed viewport region and changed
sheet text after the volley, the world hash is unchanged by presentation
inputs, and a deliberately false assertion exits nonzero with fresh captures.
Physical keyboard and mouse acceptance is recorded separately and remains
open until a person runs it.

### P4. Paredros as the second wing-glyphs consumer (ruled 2026-09-14)

Mark ruled that Paredros becomes the second consumer of `shared/wing-glyphs`
after P2 lands. Today only Mesocosm's runtime consumes it, through a trial
adapter mapping accepted carving, movement and feeding events to configured
grants. Paredros's adapter maps accepted `GameState` events, combat, injury,
treatment, equipment and terrain outcomes, to grants in the same shape, and
the P2 host gains an acquisition journal panel beside the subject sheet.
Glyph marks drawn in the scene are a later layer on the P1 producer's shared
depth attachment, as the bench's spatial glyph preview already is on the
Section's; P1's contract must not close that off. GlyphCSS's palette-as-
shading idea is the presentation twin of "one glyph, one effect" and is a
reference for that layer, not a dependency.

**Assessment, 2026-09-14 (P2 landed).** Mesocosm's consumer is
`mesocosm-runtime::glyphs`: an opt-in `GlyphRules` (canon spec, individual,
organism, unlock thresholds, event-to-glyph grants) that a `Trial` enables,
producing a `GlyphReading` holding a `wing_glyphs::Journey` plus evidence
records. It reads accepted history, changes no world fact, and grants no
durable divinity; replay is deterministic and a control change does not
transfer the collection. Paredros mirrors that shape over its own accepted
events. `GameEvent` already carries the needed kinds: `VolleyResolved`,
`Injured`, `Rested`, `Took`, `ItemAttached`, `MotionAdvanced`, `Moved`,
`Died`.

**Scope.** A `paredros_world::glyphs` reading (world crate, because the
evidence is accepted `GameState` history, not presentation): `GlyphRules`
for one subject with event-kind grants, built from `game.events()` and
advanced as new events are accepted, with the same non-durable, opt-in
posture as Mesocosm's. The session host gains an acquisition journal panel:
glyph, display mark, effect id, provenance kind and the accepting tick, in
first-acquisition order, plus eligibility. `Died` ends the reading for that
subject; succession does not carry it. Reincarnation, wishes, divine
spending and durable journey persistence stay open, as they do in Mesocosm.

**Done when:** a scripted volley, injury, rest and dressing pickup produce
the configured grants in acceptance order with exact event provenance;
rebuilding the reading from the saved and reloaded `GameState` yields an
identical journal; a control change to another subject starts an empty
reading; the panel shows the journal and the smoke asserts its text; the
world hash is unchanged by enabling the reading; and Mesocosm's 16 shared
kernel tests and its runtime tests still pass untouched. P1 and P2 added no
wing-glyphs dependency; P4 adds it to `paredros-world` only.

### Names ruled 2026-09-14

- **taproot** is genet-probe's new name (the organ a plant grows to find
  water: selectors, the Automatable and Driveable seam, the text scenario),
  and **mesquite** is the scenario lane's (the plant that lives by that
  root), moving from `shared/wing-scenario` into mere's cambium family
  because graphshell, turnstone and signalman already consume the probe.
  **tamarisk** is banked for a later extension. Both crates.io-free at the
  ruling; taproot is claimed by a real publish at 0.1.0.
- **isometer** is the game-world scene family (the wing-scene extraction),
  with lens, render and mesh as its components rather than Mesocosm-owned
  path dependencies; **isomere** is the wing-unique GUI layer, scene and
  host chrome, overlays, graphs and other applications of cambium that
  belong to the wing and not one product. Both crates.io-free at the ruling.
- **hagioglyph**: glyphs that vary over time by world criteria, a canon
  revision driven by a world period or a promoted event, with acquisitions
  keeping their founding evidence under the revision they were acquired in.
  A Paredros lane under the remembrance plan once isometer lands, so marks
  in the scene have a depth to sit on. crates.io-free at the ruling.

## Ownership and verification

Terra and Luna are the implementation lanes Mark chose. Each runs on opus,
one at a time, and reports exact files, supported operations and remaining
joins. The root agent reviews integration and runs every check in the
foreground itself; a lane's own claimed test count is not a receipt until
the root reruns it.

Stage exact owned paths under `paredros/` only. Mesocosm's benchmark and
creator files are owned by a concurrent lane and are never swept.

## Open decisions for Mark

- **Probe lane promotion (ruled 2026-09-14).** Mesocosm's
  `app/bench/probe.rs` wraps scenario, receipt, capture and exit code inside
  the Cambium host, and Paredros is the second consumer. Mark ruled: promote
  first, into an Isometry `shared/` path crate beside `wing-glyphs`
  (working name `wing-scenario`), consumed by Mesocosm and Paredros with no
  Mere commit or pin bump; it moves to mere once a non-wing consumer
  appears, mirroring the appearance-crate ruling. P3 is therefore P3a, the
  extraction with Mesocosm's bench retargeted onto it and its acceptance
  scenarios unchanged, then P3b, the Paredros scenario over the session bin.
- **Bin name.** The new host is the product session, not a probe. A name is
  a naming round, not a session default.
- **P1 is the duplicate to retire (2026-09-14).** The Mesocosm depth-sorting
  lane agreed that the shared-depth scene should become one wing crate with
  body inputs decoupled from the Mesocosm `World`, and is putting that to
  Mark as its own lane with P1's five requirements (decoupled bodies, one
  depth attachment with an orthographic camera, the glyph batch on that
  depth, Bench A's queries, the producer wrapper) as its first
  done-conditions. Section is 4,569 lines across 23 files with 31 `World`
  references, so decoupling is a lane, not a lift. Until that crate exists
  P1 stays local; when it lands, P1 becomes a thin adapter over it.
  *Ruled later the same day:* the crate is `shared/wing-scene`, beside
  wing-glyphs and wing-scenario, founded as its own lane in Mesocosm's
  presentation plan with P1's five requirements as its done-conditions;
  the extraction starts after wing-scenario landed (it did, cc7828f). P1's
  retarget is a Paredros lane once that API is messaged over.
- **Retirements.** When P2 covers them, the timed-action bin and the
  body sheet's private `EquipmentSession` are candidates for archival with
  rationale. Neither is retired by this plan.
- **Hybrid focus policy** and the other open decisions in the presentation
  plan stay Mesocosm's.

## References

- [PolyCSS](https://github.com/layoutit/polycss), MIT: one CSS-transformed
  DOM leaf per polygon. Prior art in the general model plan §6.2 and the
  presentation plan; excluded as a dependency.
- [GlyphCSS](https://github.com/apresmoi/glyphcss), MIT, a PolyCSS fork:
  keeps the mesh math and parsers, replaces the paint backend with one
  character grid written to a single `pre` element. Wireframe, solid, voxel
  and ink modes; named glyph palettes double as Lambert shading ramps;
  renders only when camera or scene changes. Relevant to the glyph canon in
  the general model plan §7.4 and the bench's stroke and spatial glyph
  work: a palette of marks standing in for light, depth and material.
  Recorded as prior art in the presentation plan on 2026-09-13; not a
  dependency.
- Bench A and Bench B in the presentation plan supply the producer and
  viewport contracts this plan consumes.

## Progress

- **2026-09-13:** founded after the renderling-tenant proposal was rejected.
  Pins ruled. No code yet.
- **2026-09-14, P0 landed.** Manifests only. The Paredros workspace check
  passes all-features/all-targets with one revision per git family
  (genet `101d9e9a`, netrender `3961aca9`, mere `4f4de1d0`); parley,
  genet-taffy and ipc-channel resolve from genet. A crates.io `taffy 0.10.1`
  remains through `servo-malloc-size-of`, identically to Mesocosm's lock.
  132 world tests and 6 native handler tests pass, rerun by the root; the
  timed-action window smoke presents the J1c values with text shaping intact
  through the patched parley. Logs and capture under
  `Code/.tmp/paredros-genet-host-20260913/`. The first P0 attempt failed
  on the dropped parley patch; the correction is recorded under Pins.
- **2026-09-14, P1 landed.** `paredros_client::producer` (root plus
  `camera`, `bodies`, `handle`, `scene`, with test-only `fixture`, `harness`
  and `tests`; every file under the ceiling). `SceneModel` holds one
  `Session` and the played subject behind `SceneHandle`, an `Rc<RefCell>`
  the P2 host will share; `SceneProducer` implements the rootstock
  `TextureProducer`: bodies as live instances first, then the tracer's
  `encode_with_depth` into the same colour and `Depth32Float` attachments
  with one `clip_from_world` from `CameraPolicy`'s orthographic slab.
  Unchanged inputs return `None`; suspend and retire drop GPU targets and
  keep the per-`(subject, revision)` mesh cache. Declared limits: parts are
  declared-extent solid boxes under one material (anatomies carry no voxel
  data); body scale is a presentation guess of 0.25; the CPU pick ignores
  terrain. Eight headless GPU tests prove nearer-body ownership of shared
  pixels, terrain occlusion with a bodies-only control, severance without
  re-upload, fractional-pose displacement within a pixel, no-op frames, and
  pick agreement; 45 client lib tests, 132 world tests, 6 native tests and
  the clean workspace check were rerun by the root. Captures and logs under
  `Code/.tmp/paredros-genet-host-20260913/p1-*`. Two findings for Mesocosm's
  lane: the tracer paints pure black at a zero-distance hit on the slab's
  front wall (visible as a block over the keeper in the terrain capture),
  and world seed 7 never buries the keeper, so terrain occlusion is proved
  as one mechanism on the target. The module is gated behind `r1-proof`
  only because `mesocosm-lens` and `modulus` are optional there; that gate
  is a wart to remove when the renderling bins retire under L7.
- **2026-09-14, P2 landed.** The `session` bin (working name) runs the
  Cambium winit host with a viewport leaf over `SceneProducer`, a subject
  sheet, an equipment panel and a status panel, all reading one `Session`.
  `SceneModel` now holds that session in a `Held` slot, plain or wrapped in
  `TimedActionSession`, so the charge grammar and every panel share it.
  The seed-7 fixture moved to `paredros_client::session_fixture`; the
  producer tests re-export it and the timed-action bin is untouched.
  Controls: WASD, arrows, Space or held left mouse, E, R, I, J, Ctrl+S,
  Ctrl+L, Esc, plus DOM buttons for the same intents and per-part and
  per-item selection; a viewport click resolves through the leaf's content
  box to the pick. The smoke runs the volley, injury, rest, dressing, save,
  move and load inside the host and captures the presented document; the
  root reran it and inspected the frame. 45 lib, 6 native and 132 world
  tests and the clean workspace check were rerun by the root. Two Cambium
  host limits are recorded rather than worked around: rootstock routes key
  presses only, so held movement is a latch refreshed by auto-repeat and
  charging is a toggle on the keyboard (the mouse carries down and up);
  and function keys lower to one `Other` variant, so save and load are
  Ctrl+S and Ctrl+L. The leaf is a fixed 720 by 440 box. The sheet's bounds
  field is empty for the picked part; that projection is paredros-world's.
  Physical keyboard and mouse acceptance remains open.
- **2026-09-14, P3a landed.** `shared/wing-scenario` (working name) is the
  promoted scenario lane: `Lane<P>` over a `Product` trait whose hooks
  supply the sheet, snapshot fields, event drain, default capture path and
  optional act, busy, viewport, target point, opacity sheet, cost
  observation and app-specific steps; `capture_path`, `Viewport`,
  `PixelCheck`, `Costs` and the shared verbs (cost, input-text, resize,
  remember, zoom, opacity, same/more, captures-differ, pixel-change checks)
  live there. Mesocosm's `bench/probe*.rs` is a thin adapter keeping the
  `Lane` API, so `bench.rs` and `probe_state.rs` are byte-unchanged; its
  Mesocosm-only selectors, sheet, transform and snapshot fields stayed
  local. 12 shared tests, Mesocosm's 158 genet tests, the workspace check
  with its two pre-existing warnings, and the bench acceptance scenario
  (167 frames, 15 captures, ok) and failure scenario (exit 1) were rerun by
  the root; Paredros's workspace check is unaffected. Receipt JSON is
  structurally identical to the 2026-09-13 baseline; the value drift in
  hash, drawn parts and instance bytes comes from another lane's
  uncommitted body and trial edits, not from the extraction. The crate's
  lock is ignored like the products' locks; it has no LICENSE file, like
  wing-glyphs. The root manifest's exclude list and mesocosm's `.gitignore`
  (now ignoring `target-contact`) changed by one line each.
- **2026-09-14, P3b landed.** The session bin is the second wing-scenario
  consumer: `SessionProduct` supplies the sheet, a 25-field snapshot
  (ready, action, aim, position, step, grounded, supports, vitality, wound,
  dressings, lost-parts, target vitality and parts, selection, saves,
  loads, hash and more), an event drain over accepted `GameEvent`s, the
  `Scene` leaf as viewport, and an `act` vocabulary for keyboard-only
  intents (movement, aim, join, charge, strike, take, rest, injure, save,
  load); DOM buttons are clicked through pointer routing. The bin takes
  `--scenario`, `--receipt`, `--capture`, `--frames` and `--size`.
  `paredros/testing/session/acceptance.scenario` moves, strikes, treats,
  selects and round-trips a save with receipts under
  `testing/session/receipts/2026-09-14/`; the root reran it (86 frames,
  4 captures, ok), the failure scenario (exit 1, fresh capture), the smoke,
  the clean workspace check and the 45 lib, 6 native and 132 world tests,
  and inspected the final frame. Two act shortcuts run movement steps and
  charge ticks outright instead of the latch and the wall clock, so
  auto-repeat timing and mouse-held charging stay with physical
  acceptance, which remains open. One shared gap is recorded rather than
  worked around in `shared/`: wing-scenario has `remember`/`same`/`more`
  but no "differs" or "dropped" verb and gives `app_step` no checkpoint
  access, so the product keeps its own `mark`/`differs`/`dropped`; promoting
  those is a wing-scenario edit. The status panel clips at 1280 by 900.
- **2026-09-14, P4 landed.** `paredros_world::glyphs`: `GlyphRules` for
  one subject (canon spec, individual, subject, unlock thresholds, grants
  keyed by `AcceptedKind` over VolleyResolved as striker, Injured, Rested,
  Took, ItemAttached, MotionAdvanced, Moved and Died) and `GlyphReading`,
  built from accepted history and advanced by cursor, holding a
  `wing_glyphs::Journey` plus evidence records; `Died` ends it. Opt-in and
  non-durable like Mesocosm's: not in `GameSave`, no intent writes it, and
  `state_hash` is proven identical with and without it. Two departures
  from Mesocosm are recorded in the module: provenance uses a custom kind
  naming the event so the motif groups by event kind, and evidence carries
  no construction-time hash, because a reading rebuilt from a restored save
  must equal one opened at the start. The session host gains an
  acquisition journal panel under the sheet, fed by an authored fixture
  canon of seven marks, and the acceptance scenario asserts the journal
  grows after the strike and the rest and survives the save round trip.
  140 world tests (8 new), the untouched kernel's 16, 45 lib and 6 native
  tests, the clean workspace check, the acceptance scenario (95 frames,
  ok), the failure scenario and the smoke were rerun by the root, and the
  final frame inspected. Receipt at
  `testing/session/receipts/2026-09-14/acceptance-glyphs.json`. Open:
  glyph marks in the scene on the shared depth, a real canon, durable
  journey persistence, reincarnation and divine spending, and the world
  crate's own copy of the seed-7 fixture (the client's cannot be reached
  from the world crate; a client/world boundary question).
- **2026-09-14, P5 landed** (the two follow-ups P3b and P4 left open). The
  seed-7 fixture has one home: `paredros_world::fixtures::session`, public and
  unconditional like `three_lives`, built from this crate's own dependencies
  only. `paredros-world/src/glyphs/fixture.rs` and
  `paredros-client/src/session_fixture.rs` are both gone; `glyphs::tests` reads
  the promoted world and keeps only the host verbs it drives it with, the
  producer tests re-export it, and the client's `Fixture::scene` stays an
  extension trait in `producer/fixture.rs`. One glyph test changed meaning with
  the shared world: the target now has accepted acts of its own, so
  "another subject starts an empty journal" became "another subject reads only
  its own evidence", which proves the same binding against a busier history.
  `wing-scenario` gained the two verbs P3b kept product-local: `Product::app_step`
  is handed a read-only `Checkpoints<'_>` view of what `remember` took, and
  `differs <checkpoint> <field>...` and `dropped <checkpoint> <field>...` join
  `same` and `more` in the shared grammar, all four sharing one comparison with
  the same attributable miss line. `dropped` reads decimals, which is why the
  struck target's falling vitality needed it. The session bin's `mark`/`differs`/
  `dropped` are deleted and the acceptance scenario is `remember` plus shared
  verbs only; Mesocosm's adapter took exactly one signature edit (the extra
  `_checkpoints` parameter) and nothing else. 17 shared tests (5 new), 140 world,
  45 lib, 6 native, the clean Paredros workspace check, the acceptance scenario
  (92 frames, 4 captures, ok) and the failure scenario (exit 1) were rerun by the
  root, plus Mesocosm's `mesocosm-genet` check (no warnings) and its 158 tests.
  Receipt at `testing/session/receipts/2026-09-14/acceptance-shared-verbs.json`.
