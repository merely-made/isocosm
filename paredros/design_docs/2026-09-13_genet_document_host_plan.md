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

## Ownership and verification

Terra and Luna are the implementation lanes Mark chose. Each runs on opus,
one at a time, and reports exact files, supported operations and remaining
joins. The root agent reviews integration and runs every check in the
foreground itself; a lane's own claimed test count is not a receipt until
the root reruns it.

Stage exact owned paths under `paredros/` only. Mesocosm's benchmark and
creator files are owned by a concurrent lane and are never swept.

## Open decisions for Mark

- **Probe lane promotion.** Mesocosm's `app/bench/probe.rs` wraps scenario,
  receipt, capture and exit code inside the Cambium host. Paredros is the
  second consumer. Promote it to a shared home under mere's cambium family,
  or copy the shape locally for P3 and promote later. The consolidation rule
  argues for promotion; the pin cadence argues for a local first receipt.
- **Bin name.** The new host is the product session, not a probe. A name is
  a naming round, not a session default.
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
