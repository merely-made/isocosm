# The sim host and the wing's GUI layer

**Status:** 2026-10-28: cut from `2026-10-14_sim_host_plan.md` and `2026-09-15_isomere_plan.md`, which are archived in
`archive_docs/2026-10-28/`. The core library `isocosm-host` is built and has no product dependency (H0 to H5 and H6's functional conditions); the Mesocosm head runs on the core and isomere's `Assembly`; isomere is landed through its old M5. Open: live viewing of the endpoints (H2 step 4), the measure H6 still names, the other products moving onto the core, the mode host's acceptance, and the two deferred isomere lanes.

**For:** the embeddable sim host (`host/`, package `isocosm-host`: world, saves, binding, founding, drive and clock, the Graphshell inspection endpoint) and the products' uniform use of it; and isomere (`shared/isomere`), the wing's GUI layer and the one-host mode design (§2.4 of the old isomere plan, rulings 442 to 445). Phases H2 and H8 are the host's; I1 to I5 are isomere's (the old isomere lanes M4 and M6 are named "was isomere M4/M6" and are not the Mesocosm overlay's M labels).

Not here: H7, the family session layer (`isonetry` remade), is `session-network H7`. Every tick, frame or search-time bar, including case 966's 1.5 s, the 976 bar, the head's tempo (978) and TD2's aspiration, is `sim-execution` (its Speed bars). The host's lighting and launch-capture findings (964 to 969) are `orthographic-voxel-presentation`. The headed run of directing (old D5) is `mesocosm-overlay M4i.2`; the full in-site play is `mesocosm-overlay M4`.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- The core is a library in `isocosm-host` with no product crate in its graph; products depend on it, not the reverse (884, 892).
- A head refuses a generated world without 875's witness; the bench, a dev instrument, founds its own worlds (885, 897). The witness comes from the generator's A1 lane (888 to 891).
- The inspection shell is a Graphshell endpoint; bench worlds prove the head before 875's inputs; all three product hosts sit on isomere's `Assembly` (896 to 898). Product panels are product-owned endpoints (900 to 903).
- The core owns the clock and an explicit step; each product picks wall-clock or explicit pacing (927). The core defines its own `Press` and a keymap hook; the window still picks, and the core refuses a pick stamped with a stale world revision (954 to 957).
- The society layer is behind `isocosm`'s feature `society`; `isocosm-host` takes `isocosm` with `default-features = false` and passes `society` through a feature of its own, so a Mesocosm crate that depends on the core keeps it off (893, 894).
- Isomere's boundary (Mark, 2026-09-14): isometer never names a Cambium widget; isomere never names a voxel. Anything a second non-game consumer could want goes to mere's cambium family, not isomere. A product keeps a residue of "what is about this host", not zero.
- One host over one world save (381): the executable lives in a host workspace of its own, taking the three products' crates by path (442); a window plays one mode over the open world and switches in place, and two modes over one world on one machine are two windows (443); the save holds the sim's history and a section per mode, and a build without a mode carries its section through unread and unchanged (444); a window's access follows its participant's seat, never its mode (445); each active window contributes to one player's attention set, taken as their union (815).
- The shared default view is the 2:1 dimetric in quarter turns (382, 387); camera changes do not grant perception or select simulation fidelity.

## Phases

### H2 step 4. Live viewing of the endpoints
When the capabilities being lifted out of Graphshell's reference host land (899), the core's and the products' endpoints adopt them for live viewing. No live viewer mounts an arbitrary endpoint today.
- Done when: not stated; owed.
- Needs: case 899 (Graphshell's reference host)
- Rulings: 896, 899, 904
- From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md` H2 step 4, lines 187-189 and 82-87.

### H8. The other products on the core
Later, no phases yet; each gets phases here when it starts: Eponym's `session` on the core (founding a generated world, asks through the UI, succession in the client); the VTT over the sim (in-app campaign founding from a seeded draw); Eponym's motion adopting the core's `Clock`; the VTT on the explicit step; and H5's import of Eponym's Game13 and the VTT campaign into `WorldSave` with the product section kept as opaque bytes (only the Mesocosm `GameSave` import has a stated condition, and it is met). Pacing stays per product under 927: Eponym paces motion at 60 Hz by `Instant` and drops its backlog after eight catch-up steps, the VTT has no wall clock (consented downtime), Mesocosm uses `Clock` (rational steps, the cap defers). Unifying them is not asked.
- Done when: not stated; owed. For the import, the one stated form is H5's: "a save written by the Mesocosm head loads in the core and resumes to the same hash, and an unknown section survives a round trip untouched."
- Needs: H0 to H5 (built); for the VTT, `vtt-overlay V2`; for Eponym, `eponym-overlay`
- Rulings: 927, 954 to 957
- From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md` "Later", lines 317-320; H3 pacing, lines 129-134; H5, lines 230-234.

### I1. Mesocosm's main binary off the hand-rolled chrome (was isomere M4)
Mark deferred this on 2026-09-15 because Mesocosm's chrome lays out in physical pixels against the shared host's logical ones, which rebases its capture set. The window and frame loop are already `Assembly`'s; what is left is deleting `chrome.rs` and its five panel adapters. When it runs it answers the two questions the assessment left: what replaces the played receipt's `frame_graph` block behind a producer, and whether the scenario grammar's `assert text` becomes a host-DOM assertion with the driver moved onto a mesquite lane.
- Done when: "Done when Isometry's and Paredros's hosts are the shared assembly plus a product impl. Mesocosm's main binary ... moves off `chrome.rs` with its five adapters deleted" (the first sentence is met per the old status; the Mesocosm half is open).
- Needs: the round that moves Mesocosm's main binary; the capture-set rebase
- Rulings: Mark, 2026-09-15 (deferred)
- From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` §4 M4, lines 224-234, and §5 decision 2, lines 251-259.

### I2. Dev tile as a frisket pane and one receipts panel (was isomere M6)
"Mark deferred all of M6 with the migration" (2026-09-16): it waits whole for the round that moves Mesocosm's main binary, so the panel lands in all three real hosts at once.
- Done when: "Done when Mesocosm's dev tile is a frisket pane and all three hosts show the same receipts panel from mesquite's lane."
- Needs: I1
- Rulings: Mark, 2026-09-16
- From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` §4 M6, lines 238-240, and Progress, lines 292-304.

### I3. Mode host acceptance
The mode host runs in `host/` over one native session with borrowed VTT, Eponym and Mesocosm inputs, transactional saves and per-window attention. Open: the host runtime and headed two-window acceptance; the host's own gates; native attention (the bridge for a participant's attention, binding intentions stamped with participant and tick), shared-world and window lifecycle, save compatibility, and headed product binding. Inventory listing alone does not certify it. This is the one host-acceptance lane that directing's headed run, the host core binding and the mode host all pointed at.
- Done when: "switching preserves world, subject and embodiment; knowledge follows the seat"
- Needs: `mesocosm-overlay M4i.2` (the headed host run, was directing D5); `mesocosm-overlay M4` (the host lands with Mesocosm's full in-site play on the old reading); rulings 815, 824, 826
- Rulings: 442 to 445, 815, 824, 826
- From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` status and §2.4, lines 5, 281-290, 306; `archive_docs/2026-10-28/2026-09-26_body_binding_plan.md` Progress, lines 455-479 and 724 (the host core binding item, merged here).

### I4. Opaque VTT product-game actuation through the host
- Done when: not stated; owed.
- Needs: `vtt-overlay V2`; I3
- From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` status, line 5.

### I5. Pin and Unpin controls
Cambium controls connected to the logged window-attention command, using the current seat-filtered native cut; ambiguous event handles are withheld. Written as source only under 826 (no compiler run, test, fixture adjustment, tuning or headed receipt), and the path the old plan names is gone, so the first step is finding where the controls now are.
- Done when: not stated; owed.
- Needs: I3; 826
- Rulings: 815, 826
- From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` Progress 2026-10-11, lines 273-279.

## Deferred and parked
- **Mode host carries only Mesocosm's mode** until overlays run on the sim (reading, not ruled): the VTT's and Eponym's modes join as `vtt-overlay` and `eponym-overlay` land. Waits on those plans. From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` lines 182-186; rulings 194, 231.
- **Examiner and incarnation controls** (design refinement of 2026-09-28, not implemented): a construction examiner connecting source parts and relationships to capabilities, repertoire, proficiency and conditions; a process examiner separating supporting requirements, cost contributions, accepted effects, causal history and the actor's beliefs; incarnation controls with granular configured or randomized carryover (406), consequences explained before commitment. Waits on `wing-phenotype-contract P1` (construction description) and `wing-phenotype-contract P4` (carryover). From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` §2.4, lines 154-170.
- **Upstream to cambium instead of isomere**: the private `button()`/`field()` helpers, the titled overlay panel if `overlay_surface` and `summary_body` do not cover it, the two parley font loaders, `atlas_labels.rs` if graphshell's atlas callouts want it. Rule: "if a second non-game consumer could want it, it is Cambium's." Waits on a mere lane. From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` lines 194-202.

## Questions for Mark
- **The still-open mode-host design (§2.4)**: build-profile packaging; session and mode-switch lifecycle; where the shared presentation/knowledge contract's type lives (reading, not ruled: the access type sits in `isocosm-overlay` beside the attention set, and under replay it shapes play, not secrecy); perspective camera support (isometer exposes an orthographic slab camera; perspective is future scene work). Mesocosm's opening view is CP1's comparison (387, 388), which `orthographic-voxel-presentation` carries. From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` lines 141-192; rulings 381, 382, 384, 387, 388, 442 to 445.
- **H6's measurement**: H6's done-condition says "frame pacing and pointer picking in the viewport leaf are measured against today's loop." "Today's loop" was the hand-rolled loop the head has since left, and no measurement is on record; the old plan gave no alternative. Is the measurement still wanted, and against what? From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md` H6, lines 236-247.

## Findings
- 2026-10-28: `host/Cargo.toml` has `default = []` with features `fixtures`, `endpoint` and `society`; there is no `ui/` and no desktop shell. The old plan's finding that `isocosm-host` depends on the products is history. `host/src` holds `world`, `save`, `binding`, `founding`, `drive/{clock,input}`, `endpoint`, `window`, `native_attention`, `access`.
- 2026-10-28: the Mesocosm head runs on the core and `Assembly` (`mesocosm-genet/src/lib.rs` says so). `chrome.rs` survives as a "compatibility raster adapter for the retained panel/GPU reference tests", and `review`, `vitals`, `dev`, `succession` and `hud` still import it; `played.rs` still carries the `frame_graph` receipt block; the dev tile is still a hand-walk of a `TileTree` in `dev.rs`. This corrects the old isomere status, which said the main binary had not moved.
- 2026-10-28: the executable-location ruling (442) is met: `host/` is a workspace of its own.
- 2026-10-28: `host/src/ui/` does not exist, so the old Pin/Unpin path is stale. `window.rs` carries `window_attention_request` for `AttentionChange::Pin/Unpin`; no Cambium control for it was found.
- 2026-10-28: the old H6 clause "a default launch founds a generated world" is met per the old status (A1 passes and ordinary new worlds select A1, case 961); only the pacing measurement is open.
- 2026-10-28: no live viewer mounts an arbitrary endpoint; `g4_sessions` writes a static HTML session receipt and Turnstone hosts only Knot's endpoint.
- 2026-10-28: the old host plan's H7 draft done-condition and its first implementation gates now live in `session-network`; this plan holds none of them.

## Progress
- 2026-10-28: cut from `2026-10-14_sim_host_plan.md` and `2026-09-15_isomere_plan.md`.
