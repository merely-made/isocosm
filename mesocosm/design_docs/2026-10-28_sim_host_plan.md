# The sim host and the wing's GUI layer

**Status:** 2026-10-28: cut from `2026-10-14_sim_host_plan.md` and `2026-09-15_isomere_plan.md`, which are archived in
`archive_docs/2026-10-28/`. The core library `cosmograph-host` is built and has no product dependency (H0 to H5 and H6's functional conditions); the Mesocosm head runs on the core and isomere's `Assembly`; isomere is landed through its old M5. Open: live viewing of the endpoints (H2 step 4), the measure H6 still names, the other products moving onto the core, the mode host's acceptance, and the two deferred isomere lanes.

**For:** the embeddable sim host (`cosmograph-host/`, package `cosmograph-host`: world, saves, binding, founding, drive and clock, the Graphshell inspection endpoint) and the products' uniform use of it; and isomere (`shared/isomere`), the wing's GUI layer and the one-host mode design (§2.4 of the old isomere plan, rulings 442 to 445). Phases H2 and H8 are the host's; I1 to I5 are isomere's (the old isomere lanes M4 and M6 are named "was isomere M4/M6" and are not the Mesocosm overlay's M labels).

Not here: H7, the family session layer (`isonetry` remade), is `session-network H7`. Every tick, frame or search-time bar, including case 966's 1.5 s, the 976 bar, the head's tempo (978) and TD2's aspiration, is `sim-execution` (its Speed bars). The host's lighting and launch-capture findings (964 to 969) are `orthographic-voxel-presentation`. The headed run of directing (old D5) is `mesocosm-overlay M4i.2`; the full in-site play is `mesocosm-overlay M4`.

## Decisions

- **D1, the games stay separate; what is shared first is reading any game's world (2026-11-12), refines 381:** "i'm ok with keeping them separate! for a cosmograph producer in a frame, i guess it would be important to be able to read any of the games' saves/worlds, most of all. that to me felt like an intuitive base for the rest of the games' code, if we could isolate and hotswap that bit, like wit or lua or a data format, whatever. as a host, i guess that's possible; as a lib or producer/supplier for the games, also possible". The three programs stay. The base every game stands on is one piece that opens any game's world or save, isolated behind a boundary that can be swapped (Stack 13's contract shape, a script, or a data format), whether it runs as a host or is linked as a library. Raised by the brief `design_docs/2026-11-12_sim_host_brief.md`.

  Links: area Host and network · draws on 381, 442 · plans sim-host · summary The three programs stay separate; the shared base is one swappable piece that reads any game's world, as host or library.

- **D2, one world opened by games in turn comes first (2026-11-12):** "One world opened by games in turn (Recommended)". Of the four kinds of linking in the brief (one format, one world in turn, one world at once, identity across worlds), the second is first: a world grown in one game is opened by another.

  Links: area Host and network · draws on 381 · plans sim-host · summary The first linking between games is one world opened by different games in turn.

- **D3, learning carries with the lineage across worlds and with the persona (2026-11-12), refines 1056:** "1 + 3 (you deserve a cosmodex!)", choosing "With its lineage when it crosses worlds" and "With your persona". A lineage that founds a fork or crosses into another world brings what it learned; what creatures learned from a player's play also travels with that player's persona as a portable record. "Cosmodex" is Mark's word for that record; it is not yet a settled name (Naming 3).

  Links: area Directing and play · draws on 1051, 1053, 1056, 1227 · plans sim-host · summary A creature's learning travels with its lineage across worlds and with the player's persona as a portable record.

- **D4, replicated worlds need the significant facts to agree, not every byte (2026-11-12):** "i wouldn't tear off my leg for this. just seems nice to have. if replicated worlds didn't resolve byte for byte identically, then i wouldn't be bothered as long as the significant facts/entities/events don't diverge." Byte-identical replay across machines is wanted, not required. What must hold between replicas is the significant record: which entities exist, the facts about them, and the events that happened. Pinned hashes stay as each machine's own regression check. This lifts the bar that kept float and GPU work (Burn, Rapier, fields) out of anything a replica sees; what counts as significant is not yet defined.

  Links: area Host and network · draws on 381 · plans sim-host · summary Replicas must agree on significant facts, entities and events; byte-identical replay is wanted but not required.

- **D5, the family owns audio, physics, fields, models and transport (2026-11-12):** Mark, naming the owner of each thing listed as missing: audio "firewheel"; physics "seiche"; "rapier, nexus (things to harvest perhaps), kiss2d/3d as a tenant renderer if needed"; models "distillery, esp, burn, athanor"; peer-to-peer "p2mere", with iroh "a carrier"; and on animation, "css animation is available if it comes down to that ... but procedurally solving animation/rendering in the cheapest, most modular, most reactive way we can would be pretty cool. i don't care about visual fidelity; i care more about performance, interactivity". The games take these from the family repositories and do not build their own.

  Links: area Host and network · draws on 381 · plans sim-host · summary Audio is firewheel, physics and fields are seiche, models are distillery and esp, transport is p2mere over a carrier; animation is procedural and cheap, fidelity second to performance.

## Constraints

- The core is a library in `cosmograph-host` with no product crate in its graph; products depend on it, not the reverse (884, 892).
- A head refuses a generated world without 875's witness; the bench, a dev instrument, founds its own worlds (885, 897). The witness comes from the generator's A1 lane (888 to 891).
- The inspection shell is a Graphshell endpoint; bench worlds prove the head before 875's inputs; all three product hosts sit on isomere's `Assembly` (896 to 898). Product panels are product-owned endpoints (900 to 903).
- The core owns the clock and an explicit step; each product picks wall-clock or explicit pacing (927). The core defines its own `Press` and a keymap hook; the window still picks, and the core refuses a pick stamped with a stale world revision (954 to 957).
- The society layer is behind `cosmograph`'s feature `society`; `cosmograph-host` takes `cosmograph` with `default-features = false` and passes `society` through a feature of its own, so a Mesocosm crate that depends on the core keeps it off (893, 894).
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

### H9. The world-graph viewer
A standalone Cosmograph viewer (case 1038): a native Graphshell client of the core's inspection endpoint, on a seed or a save. The endpoint grows a graph projection (nodes for regions, sites, populations, entities of note and groups; edges for routes, kin, relations, membership and claims), with zoom opening clusters and filters narrowing by domain; a timeline advances, scrubs back by replay without changing the world, and forks a branch at any past tick (Record 4).
- Done when: "from a seed nobody chose, the viewer shows the world's regions and routes, zooms into a place's residents, kin and groups, advances time and watches them change, scrubs back to an earlier tick and sees the earlier graph, and forks a branch there whose replay is its own"; one capture per step.
- Needs: the rename (case 1037); the region lane's readings (World 10)
- Rulings: 1038, 1048, 896, 899
- From: case 1038; founding, branching and drawing ruled in case 1048.

### H8. The other products on the core
Product-host and import acceptance remain here: Eponym's `session` on the
core (founding a generated world, asks through the UI, succession in the
client); the VTT over the sim (in-app campaign founding from a seeded draw)
and its explicit step; and H5's import of Eponym's Game13 and the VTT campaign
into `WorldSave` with the product section kept as opaque bytes (only the
Mesocosm `GameSave` import has a stated condition, and it is met).

**Pacing inventory, 2026-11-04:** the core already supplies rational `Clock`,
round/simming `Pacer` and ordered worker/in-step `Ticker` in
`cosmograph-host/src/drive/{clock,round}.rs`. A round authorizes sixty
foreground steps per real second, scaled world time and a background minute;
simming uses the tick-rate dial. Eponym's session already consumes `Pacer`
and `Ticker`: `Instant` measures elapsed host time, rather than providing a
separate pacing engine. Its product policy still limits local drawn-motion
work to eight steps times speed per frame and lets excess local steps go;
due background minutes remain owed and run one at a time
(`eponym-client/src/round.rs`). Mesocosm uses the core `Clock` through its
driver, whose cap defers steps. The VTT's consented downtime stays explicit.
Product pacing policy remains separate from rendering demand under 927;
retaining a picture neither advances native time nor replaces a minute's
admitted foreground handbacks. Shared presentation receiving continues in
[scene composition C6](2026-11-04_scene_composition_plan.md#c6-shared-qualified-output-and-native-frame-receiving).

- Done when: not stated; owed. For the import, the one stated form is H5's: "a save written by the Mesocosm head loads in the core and resumes to the same hash, and an unknown section survives a round trip untouched."
- Needs: H0 to H5 (built); for the VTT, `vtt-overlay V2`; for Eponym, `eponym-overlay`
- Rulings: 927, 954 to 957
- From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md` "Later", lines 317-320; H3 pacing, lines 129-134; H5, lines 230-234.

### I1. Mesocosm's main binary off the hand-rolled chrome (was isomere M4)
Mark deferred this on 2026-09-15 because Mesocosm's chrome lays out in physical pixels against the shared host's logical ones, which rebases its capture set. The window and frame loop are already `Assembly`'s; what is left is deleting `chrome.rs` and its five panel adapters. When it runs it answers the two questions the assessment left: what replaces the played receipt's `frame_graph` block behind a producer, and whether the scenario grammar's `assert text` becomes a host-DOM assertion with the driver moved onto a mesquite lane.
- Done when: "Done when Isocosm's and Paredros's hosts are the shared assembly plus a product impl. Mesocosm's main binary ... moves off `chrome.rs` with its five adapters deleted" (the first sentence is met per the old status; the Mesocosm half is open).
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
The mode host runs in `cosmograph-host/` over one native session with borrowed VTT, Eponym and Mesocosm inputs, transactional saves and per-window attention. Open: the host runtime and headed two-window acceptance; the host's own gates; native attention (the bridge for a participant's attention, binding intentions stamped with participant and tick), shared-world and window lifecycle, save compatibility, and headed product binding. Inventory listing alone does not certify it. This is the one host-acceptance lane that directing's headed run, the host core binding and the mode host all pointed at.
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
- **The still-open mode-host design (§2.4)**: build-profile packaging; session and mode-switch lifecycle; where the shared presentation/knowledge contract's type lives (reading, not ruled: the access type sits in `cosmograph-overlay` beside the attention set, and under replay it shapes play, not secrecy); perspective camera support (isometer exposes an orthographic slab camera; perspective is future scene work). Mesocosm's opening view is CP1's comparison (387, 388), which `orthographic-voxel-presentation` carries. From: `archive_docs/2026-10-28/2026-09-15_isomere_plan.md` lines 141-192; rulings 381, 382, 384, 387, 388, 442 to 445.
- **H6's measurement**: H6's done-condition says "frame pacing and pointer picking in the viewport leaf are measured against today's loop." "Today's loop" was the hand-rolled loop the head has since left, and no measurement is on record; the old plan gave no alternative. Is the measurement still wanted, and against what? From: `archive_docs/2026-10-28/2026-10-14_sim_host_plan.md` H6, lines 236-247.

## Findings
- 2026-11-04 (H8): current Eponym imports the shared `Pacer` and `Ticker`
  (`eponym/crates/eponym-client/src/round.rs:10,69-74`); its local motion cap
  and owed background minutes are distinct (`:134-153`). The former
  `Instant`-versus-`Clock` inventory hid that shared mechanism. Native
  attention remains an admitted window contribution
  (`shared/cosmograph/src/attention/admission.rs`); `spatial/tier.rs`
  derives the participant's capped foreground, and `spatial/handback.rs`
  validates minute handbacks without moving matter. This source audit does
  not close H8's remaining product-host or import acceptance.
- 2026-10-28: `cosmograph-host/Cargo.toml` has `default = []` with features `fixtures`, `endpoint` and `society`; there is no `ui/` and no desktop shell. The old plan's finding that `cosmograph-host` depends on the products is history. `host/src` holds `world`, `save`, `binding`, `founding`, `drive/{clock,input}`, `endpoint`, `window`, `native_attention`, `access`.
- 2026-10-28: the Mesocosm head runs on the core and `Assembly` (`mesocosm-genet/src/lib.rs` says so). `chrome.rs` survives as a "compatibility raster adapter for the retained panel/GPU reference tests", and `review`, `vitals`, `dev`, `succession` and `hud` still import it; `played.rs` still carries the `frame_graph` receipt block; the dev tile is still a hand-walk of a `TileTree` in `dev.rs`. This corrects the old isomere status, which said the main binary had not moved.
- 2026-10-28: the executable-location ruling (442) is met: `cosmograph-host/` is a workspace of its own.
- 2026-10-28: `cosmograph-host/src/ui/` does not exist, so the old Pin/Unpin path is stale. `window.rs` carries `window_attention_request` for `AttentionChange::Pin/Unpin`; no Cambium control for it was found.
- 2026-10-28: the old H6 clause "a default launch founds a generated world" is met per the old status (A1 passes and ordinary new worlds select A1, case 961); only the pacing measurement is open.
- 2026-10-28: no live viewer mounts an arbitrary endpoint; `g4_sessions` writes a static HTML session receipt and Turnstone hosts only Knot's endpoint.
- 2026-10-28: the old host plan's H7 draft done-condition and its first implementation gates now live in `session-network`; this plan holds none of them.

- 2026-10-31 (H9): a bench-founded generated world (`Founding { sites: 12, population: 200, lineages: 6, ecology: true }`, seed 1793451363859242500, debug build) has 12 sites, 22 route entries and 1 region; its stored groups grow from 19 to 222 over 50 ticks with 72 parent and 72 child relations, and it holds 0 named individuals, locations, households and polities. The quick-start Mill Ford (`cosmograph::quickstart::session`) holds 6 named individuals, 1 location read as a hamlet of 4 residents and 1 polity, and did not change over 50 ticks (9 groups and 22 events throughout). So H9's place step is shown at site grain (resident groups and their kin) and stays open until World 10's population work brings places and factions to a generated world (case 1048).
- 2026-10-31 (H9): a branch forked by `Session::fork_at` (`shared/cosmograph/src/history.rs:363`) and left unchanged replays its main line exactly: forked at tick 15 and advanced to 40, its state hash equals the main line's at 40, because the branch key seeds nothing. A branch becomes its own only by a change made inside it (case 1048); `cosmograph-host/src/tests/graph.rs` holds both halves.
- 2026-10-31 (H9): `Session` keeps no restorable past state (`checkpoints` are witnesses, `history.rs:31-34`), so a scrub replays from genesis: `fork_at(20)` took 0.96 s against 1.85 s to advance 50 ticks on the 12-site world in a debug build, and a `Session` clone took 0.27 ms. The viewer's timeline therefore keeps one clone per tick it steps and replays only ticks it never kept.

- 2026-10-31 (H9): the board places cards at the endpoint's spiral slots and fits the spiral into the viewport, so a card's drawn width falls as the graph grows: about 70 px at 17 cards and narrower at 40, where titles such as "Site 10" and a closed site's "+N" are cut off (captures in `Code/testing/isocosm/world-graph-viewer/2026-10-31-run2/`). The board's physics does not pull related cards together, so the layout is the score's, not a force layout.
- 2026-10-31 (H9): `cosmograph-host` pins mere at `122451ca` (the landed relation-reading force and camera; first `79837737`, the remote board's move) (the landed merge of the remote board's move) and carries mere's root patch tables at that rev; the endpoint's existing mere crates moved with it from `b232631f`.
- 2026-10-31 (H9): with the board's law reading the scene's relations and a camera over fixed cards (mere stack-lift D3), node cards of 132 by 30 read at one layout unit to a pixel, and a site's residents gather around it. Of the laws, Springs rests linked cards about 170 apart, wider than a card; Energy gathers harder (116) and spreads the graph wider, so at one unit to a pixel its cards crowd within a cluster and the graph runs past the view, where the fitted view serves. The viewer opens on Springs at actual size; the panel switches the law live (Mark: "Selectable in the panel"), and the world card is linked to its regions so no law flings it away.
- 2026-10-31 (H9): at actual size some cards still overlap where the force law packs a dense part of the graph (in run 5, Site 5 with Site 7, Site 1 with Site 3, Site 4 with Site 6): the twelve sites carry 22 route edges, and the law's exclusion reads a body's radius, not a card's footprint. Zooming in separates them; not a blocker.
- 2026-10-31: the whole wing moved to one mere rev with the viewer (`122451ca`), because the root workspace reaches `cosmograph-host` through `isocosm-native`, and two mere revs in one graph made two `chirograph` crates (E0308 in `crates/isocosm-native/src/adventure.rs`). Every manifest's mere pin moved from `b232631f`, the family patch tables followed mere's at that rev (eidetic `613799af`, armillary `7eebeeb4`), and the `taffy` patch row went, as mere's own manifest dropped it; genet, murm and dramatis lock at mere's revisions.

## Progress
- 2026-10-28: cut from `2026-10-14_sim_host_plan.md` and `2026-09-15_isomere_plan.md`.
- 2026-10-31: H9's endpoint half on branch `world-graph-viewer`: the core endpoint offers a second projection, the world graph (`cosmograph-host/src/endpoint/{graph,graph_world,timeline}.rs`), with regions, sites, routes, resident groups, kin, locations and polities; closed clusters as sceno folds; domain filters; advance, scrub, branch, line switch, and a death or birth inside a branch.
- 2026-10-31: H9's viewer: `world-graph-viewer`, a binary behind `cosmograph-host`'s `viewer` feature, a Graphshell client of the core endpoint over the in-process carrier, drawn by `graphshell_host::board` on Pictograph's board in a Cambium window, with a scenario lane; the H9 scenario passes on two seeds nobody chose, one capture per step (receipts in `archive_docs/receipts/2026-10-28_sim_host_plan_receipts.md`).
- 2026-10-31: H9's layout: the viewer draws node-sized cards (132 by 30) under a camera (wheel zoom, drag pan, fitted or actual size) with the force law chosen in the panel, Springs by default; the H9 scenario adds the clustering, law and camera steps and passes (receipts).
- 2026-11-04: corrected H8's timing inventory against current `Clock`,
  `Pacer`, `Ticker` and Eponym receiving source; separated local motion caps,
  due native minutes and shared rendering continuation. No pacing or native
  authority change is implied by this documentation correction.
