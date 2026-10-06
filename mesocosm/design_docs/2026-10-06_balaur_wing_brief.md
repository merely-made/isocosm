# Balaur, read for the games wing

**Status, 2026-10-06:** the wing's half of the read is done; held for the
physics lane's findings (mere's balaur review brief, §3, "For the games
wing"), after which the two together check the rulings named below and the
forks in §4 go to Mark (ruling 603).

**Subject.** [balaurengine/balaur](https://github.com/balaurengine/balaur) at
`de0df794eee4eea223ed7efd31461044c71999f0`, MIT, "Copyright (c) 2026 Sébastien
Crozet, Dragos Daian". Read-only: `ARCHITECTURE.md`, the workspace
`Cargo.toml` and `Cargo.lock`, and four source files, fetched as pages. No
code copied; any port is its own decision with a licence row. Mere's brief
(`mere/design_docs/mere_docs/research/2026-10-06_balaur_review_brief.md`,
`ed8b67e8`) reads the same commit for Scenograph's editor and seiche's G8; this
one reads it against the wing's rulings.

## 1. Corrections to the relayed summary

Checked at the pinned commit, against the summary the physics session relayed:

- **rapier is 0.36**, not 0.35: `rapier3d` and `rapier2d` 0.36 with
  `enhanced-determinism` (workspace `Cargo.toml`). `ARCHITECTURE.md` itself
  still says 0.35.
- **Not f64 throughout.** Scripts use f64 (Rune's number); the engine and
  physics use f32, rapier's width. What holds throughout is libm: `glamx` 0.3.1
  with `libm` and `scalar-math`, rapier's scalar-math, `libm` 0.2 for every
  transcendental, and a Rune fork (`deterministic-pow`) for exact conversions.
- **The kiss3d patch** is the git branch `Ughuuu/kiss3d`
  `balaur-wrapped-surface` at `4522a762` (kiss3d 0.46.0), not `balaur-hooks`.
  GitHub reports 82 commits and 125 files over `v0.46.0` and will not render
  the diff; the per-patch account is the physics lane's.
- hecs is 0.11.1, the version ruling 481's bench measured. The toolchain pin
  is 1.98.1, the same as Isometry's.

## 2. What the wing holds, and what balaur does

### 2.1 Render: ruling 471 stands as ruled

471 reshapes kiss3d in four ways: an explicit context handle for the
thread-local singleton, a render-into-caller-targets entry with no window,
the tracer's depth as a pre-pass, and its shadow atlas and light buffer
exported. Balaur's fork does none of the first two. Its `Context` is still a
`thread_local!` singleton (`src/context/context.rs`, lines 10 to 15 at
`4522a762`), installed by `init` from a caller's wgpu instance, device and
queue. Its offscreen mode (`crates/balaur_render/src/kiss3d_backend.rs`,
`run_offscreen`) builds kiss3d's own headless `Window`
(`Window::new_headless_with_setup`) and renders into kiss3d's target, not the
caller's. So balaur is a working consumer of kiss3d 0.46 and a donor of
patterns (a prelinked shader cache keyed by link parameters, three render
modes agreeing bit for bit because rendering only observes), not the reshape.
Whether the reshape starts from balaur's branch or from upstream waits on the
physics lane's patch list.

### 2.2 ECS: hecs confirmed, the architecture differs by design

Balaur keeps hecs as the world of record: the scene tree is `Name`,
`Parent`, `Children` and `GlobalTransform` components on hecs entities, and a
single-threaded staged scheduler (First, PreUpdate, Update, FixedUpdate,
PostUpdate, SceneSync, Render, Last) runs systems in registration order, one
fixed-step accumulator draining `FixedUpdate`. That confirms 481 (hecs) and
is the opposite of 482 and 483: the wing's record is the source of truth, the
ECS only storage under the mode host's armillary schedule, and the
projection's diff comes from the record's receipts. That is the sim's
requirement, not a taste, so nothing here reopens 482 to 484. Two patterns
may serve 484's boundary crate: interpolation between the last two fixed
steps done render-side only, so no system ever reads a blend; and a
`StableId` that survives renames and reparenting.

### 2.3 Determinism: where balaur is ahead

What Isocosm has, measured at isometry `cf3b1dbb`:

- **An integer sim.** No `f32` anywhere in `shared/isocosm/src`; `f64`
  appears only in tools, probes and tests, and the five transcendental calls
  sit in the `isocosm-scale` curve fit and a probe's statistical bound. Maps
  are ordered (`BTreeMap` in 100 files, `HashMap` in one).
- **One whole-state hash.** `Simulation::state_hash`
  (`shared/isocosm/src/simulation.rs:192`) clones the state and takes SHA-256
  over its JSON (`crate::digest`, `src/lib.rs:48`). It is the equality
  instrument for replay, saves and branches, called 220 times across the
  tree, and it answers only "equal" or "not".
- **Order-free draws.** `crate::draw` keys every draw by seed, domain and
  values through SHA-256, so visitation order never changes a result.

What balaur adds:

- **A labelled digest.** `crates/balaur_core/src/digest.rs` (about 450
  lines) folds a list of labelled entries into one FNV-1a u64 per tick, floats
  hashed by bit pattern, entries keyed by stable id or name path and never by
  entity index, sources registered by plugins. `first_divergence` (line 228)
  names the first entry two runs disagree on, and `balaur replay
  --entries-at <tick>` prints them.
- **Cross-machine float identity, tested.** Per-tick digests exported on
  Linux, macOS arm64 and Windows and diffed in CI; physics determinism
  asserted between one-thread and eight-thread runs.
- **Replay discipline.** JSON Lines records (a header, then input and digest
  per tick), a snapshot ring and rollback session, and `ExternalIo`: outside
  work reports back only as recorded input, and its workers never start while
  replaying.

Where floats live on the wing: Eponym's motion (`eponym-motion`, 50 `f32`,
no transcendental calls of its own) solves over conatus, which at the wing's
mere pin `32edc2ad`, and at mere's head `8fa9bf26`, takes `rapier3d = "0.33"`
with neither `enhanced-determinism` nor libm. D18 (2026-09-21) parked "one
software `libm` everywhere and no platform intrinsics" as a note "for the day
they matter". Mere's brief records `enhanced-determinism` measured free
within run spread at 500, 2,000 and 5,000 bodies.

### 2.4 Scripting, UI, text, audio, input

- **Scripting.** Balaur's `balaur_script` is a language-neutral seam: traits
  and one neutral `Value`, every operation declared once in core and
  reachable from each language, Rune the first backend, hot reload keeping
  the old unit on a compile error. The wing has Wasmtime for mods and
  extensions (§4.7) and piccolo Lua for the VTT's system plugins; the seam's
  shape bears on how an overlay reaches the sim's verbs as handles (ruling
  10) when W5 writes the overlay contract. A pattern, not a fork today.
- **Already ruled otherwise:** egui and taffy (the wing's UI is Cambium over
  genet), cosmic-text and swash (genet's parley), rodio (audio from woodshed:
  cpal, hound), gilrs (gamepads are genet's, through the W3C Gamepad API,
  ruling 21).

## 3. Not compared

The editor as a project of the engine, the inspector from the property
registry, tweens as generated clips and prefab overrides that patch are the
mere review's slices for Scenograph, not the wing's. Balaur's rollback
netcode meets a standing rule in all three products' `CLAUDE.md` files: no
rollback netcode, speculatively (Mesocosm and Eponym) or at all (the VTT,
revisited only through a plan).

## 4. Forks drafted, held for the physics lane

Not yet put to Mark. Each is checked against the lane's findings first.

1. **A divergence locator beside the state hash.** Keep SHA-256 over the
   whole state as the address (branches and saves name it), and add a
   labelled form, entries keyed by the record's stable ids (places, bodies,
   processes), with a first-divergence report that replay and save tests
   print on a mismatch. Against: leave the hash as it is; or replace it with a
   per-tick FNV digest as balaur does.
2. **Cross-machine float identity for game-side physics.** Turn D18's note
   into a wing requirement now, for the float paths the sim hands out
   (Eponym's motion through conatus): rapier's `enhanced-determinism`,
   scalar-math and libm, carried by the physics lane's G8, with a digest
   diffed across Windows, macOS and Linux. Against: keep D18's note until
   peers verify each other's replays; or same-machine replay only. Turns on
   whether a save or a session crosses machines, which the wing has not ruled.
3. **A thread-count receipt for the shards.** §4.7 designs the ordered merge
   before the shards; balaur asserts one-thread and eight-thread runs equal.
   The shard work's receipt would assert one shard and N shards produce the
   same state hash. Against: leave it to the shard plan.
4. **Where 471's reshape starts:** from balaur's kiss3d branch, inheriting
   its web canvas, IME, wrapped surface and 2D-material screen copy, or from
   upstream kiss3d 0.46. Waits on the lane's patch list.
