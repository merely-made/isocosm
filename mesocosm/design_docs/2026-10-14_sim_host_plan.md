# The sim host: one embeddable core under three uniform product hosts

**Date:** 2026-10-14

**Status, 2026-10-14:** plan. Opened by rulings 884, 892 and 895; carries
887's first milestone. Nothing is built yet. Forks are listed at the end.

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

## Phases

**H0. Invert the dependency.** `isocosm-host` becomes a library holding
`World`, `WorldSave`, attention binding, file saves and streams, with no
product crate in its graph. The inspection shell moves to a binary that
depends on the core and the products (fork 1).
Done when the core builds and tests with no product dependency; the shell
builds with all three features and its existing tests pass; and
`--validate-save` behaves as before.

**H1. Founding.** The core founds a `Session` from a product's founding
request: preset, seed, population and the generated world's 875 witness when
there is one. Under 885 it refuses a generated world without the witness.
Done when one test refuses a witnessless generated founding; one founds a
fixture or bench world (fork 2) to the same state hash as the product's
existing path; and one round-trips the founded world through `WorldSave`.

**H2. Drive and pace.** The core's driver owns the envelope queue, the
wall-clock pacing, the per-advance step ceiling and the envelope trace. A
product supplies its translation from overlay envelopes to native commands
and its own loop hooks. Mesocosm's `Runtime` is re-expressed on it, and its
interim loop stays Mesocosm's.
Done when `mesocosm-runtime`'s tests pass unchanged; a recorded Mesocosm
trace replays to its recorded hash through the core; and a planted fault in
the translation seam is caught.

**H3. Live input.** Keys and pointer arrive from the window host. The core
dispatches them through a product keymap to envelopes, and resolves pointer
picks against the product's viewport.
Done when `mesocosm-genet`'s E, Space, click, Q, right-click and S, and the
checkpoint and board keys, reach the session through the core; a headless
input test drives them; and the bench scenarios still pass.

**H4. Saves and import.** `WorldSave` is the one envelope. The core imports
a Mesocosm `GameSave`, and later Eponym Game13 and the VTT campaign, into it
with the product section kept as opaque bytes.
Done when a save written by the Mesocosm head loads in the core and resumes
to the same hash, and an unknown section is carried untouched through a
round trip.

**H5. The Mesocosm head on the core.** `mesocosm-genet` founds, drives,
takes input and saves through the core, on `isomere::host::Assembly` (fork 3).
Done when the head boots on a bench or fixture world with a captured frame;
`--watch` resumes a save written through the core; and once 885 resolves (the
A1 lane's 890 capture path passes the joint witness) a default launch founds
a generated world. For 887, a lineage reaches the epoch boundary and the
trait board, captured.

**Later.** Eponym's `session` on the core (founding a generated world, asks
through the UI, succession in the client), then the VTT over the sim
(in-app campaign founding from a seeded draw). Each gets phases here when it
starts. The unified inspection shell keeps working throughout.

## Forks for Mark

1. **The shell's package.** A library-and-binary package cannot depend on
   products that depend on its library without a package cycle. *To verify
   in H0.* The shell needs a package of its own, or each product hosts it.
2. **Proving worlds before 875's inputs land.** Bench worlds (the dev bench
   already founds generated worlds), an authored fixture world through
   `--spread`, or wait for the inputs.
3. **Uniform means isomere `Assembly`.** Whether all three product hosts sit
   on `Assembly`, which moves `mesocosm-genet` off its own winit/wgpu loop.

## Progress

- **2026-10-14:** plan written from the runtime check and the code read
  above; rulings 884 to 895 recorded.
