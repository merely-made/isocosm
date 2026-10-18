# isomere, the wing's GUI layer: what moved out on 2026-10-18

**Why this moved.** On 2026-10-18 Mark made the old numbered rulings cases, kept as precedent, with new rules numbered by him alone, and asked every active doc to lose its stacked status banners, old progress entries, stale counters and pasted copies. This file holds, verbatim, what left the live [isomere plan](../../2026-09-15_isomere_plan.md): its stacked status banners and the Current state section that sat with them, and the Progress entries other than the four that describe the open work (the host's attention and pin connection, the M6 deferral and the combined migration).

## Status banners

**Current source, 2026-10-11 (826):** the live host now exposes Pin/Unpin
controls for its seat-filtered entities, sites and attended events. They use
the existing native per-window command path; a toggle changes that window's
contribution while native retains the participant's union. Mode memory and
existing save capture retain the contribution. This UI connection is placed
in production source and awaits verification after the remaining migration
pieces are in place. Opaque VTT product-game actuation remains unwired.

**Earlier checkpoint, 2026-10-11:** the mode host is implemented in `host/` over one
native session and shared rendering device. Borrowed VTT, Eponym and
Mesocosm inputs, transactional saves and per-window attention are applied.
Isomere and the combined host passed all-target/all-feature compiler checks;
host runtime and headed two-window acceptance remain open.

## Current state, 2026-10-11

`shared/isomere/src/lib.rs` exposes the shared sheet, viewport, examiner,
journal, status and optional host assembly. Those implemented M0 to M4
lanes are distinct from §2.4's mode host, now present as the separate
`host/` workspace. It borrows native authority and each product's inputs.

Mesocosm runs a native session through
`mesocosm/crates/mesocosm-runtime/src/runtime.rs`. D5 switched its host
under the compile gate, with certification left to the after-pass.
The earlier “waits on Mesocosm's M3” statuses and October 1 reading are
superseded. Ruling 798 opens V2 and E3 before M3 certification too.
Eponym's body path in `eponym/crates/eponym-client/src/gpu/body.rs` is
the tenant, meeting 442's renderling-retirement prerequisite.

The combined host's own gates remain: switching preserves world, subject
and embodiment, unread mode sections survive saves, and knowledge follows
the participant's seat and awareness. Native overlays do not certify them.

**2026-10-11, ruling 815:** active windows belonging to one participant
contribute to that participant's single attention set. Their pinned
subjects and close-up regions combine; closing or replacing one window
withdraws only that contribution. Modes and controls remain independent
under 443. Native play, care and seat access retain their existing owners.
The native admission/history, transactional host binding and presentation
filtering are applied in source. The latest combined host runtime and
headed two-window checks remain open.

*The dated design and receipts below remain as written (793).*

**Status, 2026-10-01:** The mode host ruled (442 to 445); waits on Mesocosm's M3. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status, reconciled 2026-09-28:** shared library exists at `shared/isomere`.
The Progress record below closes the original round at M5, with M6 deferred;
those are historical receipts, not tests rerun in this documentation pass.
The former header, "No crate founded", was stale. One host carrying the
three modes is now the design target under ruling 381; the shared assembly
alone does not implement that target.

**Status, 2026-10-01:** the mode host's location, switching, save and
knowledge contract are ruled (442 to 445, in [§2.4](#24-one-host-and-independent-presentation-choices)).
Nothing is built, and the host waits on Mesocosm's M3 for its first mode.

## Progress entries

- **2026-10-10, Q1:** current ownership and status checked against the lane
  tree under 793; dated prose retained. Documentation verification only;
  no new test, draw or headed certification.

- **2026-09-15:** designed from the inventory. Approved by Mark the same
  day with two rulings: the palette is seeded from today's exact hex so
  captures hold and tinct fills only the unpicked states; and Mesocosm's
  main-binary host migration (M4) is in this round, after M0 to M3.
- **2026-09-15, M0 founded (ab4d4c0, Paredros half; Mesocosm half in the
  working tree).** `shared/isomere` is its own workspace beside isometer,
  MPL-2.0, cambium and tinct from mere at 876320fd, nothing from isometer.
  `isomere::sheet` carries the 21 rules the bench and session sheets agreed
  on with every colour and every disputed length an `--isomere-*`
  property; `Seeds`, `Picked`, `derive` and `css_vars` reproduce both
  products' hand-picked hex exactly and derive only the unpicked states;
  `Sizes` carries the four font sizes and three box numbers that differed.
  Both sheets are now seeds plus product rules (the bench keeps 37, the
  session 14). Nothing was rebased. Receipts: 12 isomere tests; Paredros
  check clean, 45 lib tests, acceptance ok and failure exit 1; Mesocosm
  check clean, bench acceptance and spatial-coverage ok with 47 of 47
  captures byte-identical to a pre-change baseline and 32 of 32 to the
  kept 2026-09-14 set. Paredros's captures differ from the previous run at
  one pixel, (1647, 595), red channel by one count, which a same-binary
  rerun flips in both directions: pre-existing nondeterminism, recorded
  here so it is not chased again. The Mesocosm bench half (view.rs,
  probe.rs, bench.rs and the manifest row) lands once the shared manifest
  is free of the effect-pack lane's row. *Landed as de699c0 the same day.*
- **2026-09-15, M1 landed (9e0aed5).** `isomere::viewport` is the card
  the bench and the session each wrote by hand: `ViewportCard` carries a
  leaf key, a box, a label and the four things the two differed on (leaf
  class, id, description, and the bench's transform style in the leaf's
  own slot); `viewport_card` emits the scene card, the leaf with its image
  role and label, and an optional overlay; `scene_card` is the container
  alone; `error_line` emits the product's id plus the shared class.
  `Child<State, Action>` is spelled as both products' own alias, so neither
  needed a wrapper. The only DOM change is the added `.error-line` class
  the M0 sheet already styled. The Isometry overmap leaf did not move: it
  is Cambium's graph swatch in a panel, not a card, so §1's row is
  corrected and it belongs to a later milestone. Receipts: 19 isomere
  tests; Paredros 45 lib tests, acceptance ok, captures unchanged except
  the recorded flicker; Mesocosm bench acceptance ok, 15 of 15 captures
  byte-identical to the M0 set.
- **2026-09-15, M2 landed.** `isomere::examiner` is the panel the
  session's subject sheet and the bench's parts examiner each wrote by
  hand: `ExaminerRow` carries an id, a label, a condition class and a
  selected flag; `ExaminerModel` adds the heading, the name-value
  readings, the schematic slot M5 waits on, and the two things the
  products differed on (the container class, and the bench's "choose a
  part" note, which rides under a generation-change row and so is the
  product's call rather than "no rows"). Chips are `button.part` with
  `aria-label` and `aria-pressed`; readings are `.field` / `.field-name`
  / `.field-value`. A condition class replaces the selected class rather
  than joining it, which is what the session did, and `aria-pressed`
  carries the truth either way. Cambium's `detail_panel` was read and not
  adopted: its `.detail-row` spans are not what the M0 sheet styles, and
  adopting them would restyle both columns. The only DOM change anywhere
  is the `aria-pressed` the session's chips now carry and the bench's
  already did. Isometry's `sheet.rs` did not move: it is a character-sheet
  overlay of `.sheet-row` lines with no palette and no addressable part,
  so §1's examiner row is corrected for it as the viewport row was for
  the overmap in M1. The test file split at the ceiling into an M0 roof
  with M1 and M2 siblings. Known inert row: isomere's `[patch]` mirrors
  Mesocosm's `genet-taffy` entry, which nothing in this crate's graph
  consumes, so cargo reports it unused on every run. Receipts: 25 isomere
  tests; Paredros check clean, 45 lib tests, session acceptance ok with
  captures unchanged except the recorded (1647, 595) flicker and
  `selected-part == 3` intact; Mesocosm library check clean, release
  build, bench acceptance ok with 15 of 15 captures byte-identical to the
  M1 set and every receipt field unchanged.
- **2026-09-15, M3 landed.** `isomere::journal` is the ordered rows
  Paredros's acquisition journal and Mesocosm's trait board each wrote by
  hand: `JournalRow` carries a headline, an optional mark two spaces
  after it, a founding line that never moves, an optional live line, and
  the board's selection cursor. `JournalClasses` extends the
  `None`-means-shared idiom to class names, because the two sources are
  styled by different sheets (the session by M0's, the board by its own
  `board_css` in a netrender raster M4 and M6 own), so the board became
  the shared row without moving a pixel. `isomere::status` is the lines
  nobody clicks: `status_line` (the product's id plus the shared class
  and `role="status"`), `status_panel`, and `help_line` over a declared
  `Keymap`. The keymap is the milestone's real work: `Keymap::command` is
  what a key handler dispatches through and `Keymap::help` is the
  sentence, so a binding cannot move without the line moving with it. It
  is generic over the product's press type, because Cambium's `Key` and
  cambium-rootstock's are different types and taking rootstock would
  pull wgpu into a crate that emits elements. Cambium's `command_surface`
  and `sectioned_list` were read and not adopted. Paredros's key handler
  and its control-help line are one declaration; Isometry's eight plain
  board verbs are `isometry_views::KEYMAP`, which `key_intercept`
  dispatches through and the side panel's crib is read off. Two §1 rows
  are corrected: Isometry's message log is five flat `.roll-line` divs
  sharing the dice log's rule, not a journal, and its `.side-status`
  waits for M4. The M0 sheet gained `.journal-founding, .journal-live` on
  the existing `.field-name` rule and nothing else; §2.1's claim that it
  already carried journal-row and status-line rules was wrong. The root
  workspace excludes `shared/isomere` as it does `shared/isometer`.
  Receipts: 41 isomere tests; Paredros check clean, 45 lib tests,
  acceptance ok with captures unchanged except the recorded flicker;
  Mesocosm check clean, release build, 46 mesocosm-views tests, bench
  acceptance ok with 15 of 15 captures byte-identical to the M2 set;
  Isometry workspace check clean under all features and all targets,
  370 tests. Newly recorded: Paredros's `acceptance-opened.png` also
  flickers over a text region near x 907 to 1024, y 148 to 297 logical,
  which a same-binary rerun flips both ways; pre-existing, not chased.
- **2026-09-15, M4 landed in half: the assembly and Paredros.**
  `isomere::host` is the seven `HostHooks` closures with the five things
  all three products filled by hand filled once: producer registration
  for viewport leaves, the mesquite lane in `after_frame` and
  `close_request` with the deferred-close redraw, capture arming, the
  error line resolved from the product's own scene error then the first
  producer error among its leaves, and the frame profile line. `Product`
  is what is left, four associated types and fifteen defaulted methods,
  and `ScenarioLane` is the two calls every host already made on
  `mesquite::Lane`, blanket-implemented for it so a product hands its
  lane over as itself rather than through a forwarding newtype.
  `Capture` is Isometry's bounded capture-directory policy with the env
  names, filename and prefix lifted into `CaptureNames`; it writes
  through `mesquite::write_png`, so the wing still has one PNG encoder.
  The module is behind a `host` feature because the winit host drags
  wgpu in and the M1 to M3 panels must stay payable by a view crate.
  Paredros's session is the assembly plus an 85-line product impl, its
  probe lane newtype deleted and its smoke lane an ordinary
  `ScenarioLane`; no DOM changed and no view file was touched.
  `cambium-genet-winit-host` does not re-export `RootView`, so isomere
  names `cambium-rootstock` as well; a one-line upstream re-export would
  retire that, recorded for §3. Receipts: 41 isomere tests by default and
  49 with the host feature; Paredros check clean, 45 lib tests,
  acceptance ok and the failure scenario exit 1, captures matching a
  same-tree pre-change baseline except one-count single-channel pixels at
  (1647, 595) and a newly seen (1610, 922), both flipping direction
  between runs; Mesocosm untouched, check and release build clean, bench
  acceptance ok with 15 of 15 captures byte-identical. Newly recorded:
  the session's charge is wall-clock timed, so a slow frame can extend a
  latch and move the whole run's world state, seen once in three runs.
  Not an M4 regression, and a real fragility in that acceptance.
- **2026-09-15, Mesocosm's main binary held for Mark.** The M4 lane took
  before-baselines and stopped, because the move forces four rulings.
  (1) The chrome is laid out in physical pixels and the shared host in
  logical ones, so at this machine's scale factor of 2 every panel
  constant is half the size it would need and the glyphs rasterise
  differently: the captures rebase by construction, which M0's ruling
  that captures hold does not cover. (2) Deleting `chrome.rs` deletes the
  played receipt's whole `frame_graph` block, which is the opaque-tenant
  envelope and has no source behind a `TextureProducer`. (3) The
  scenario grammar's `assert text` reads the six lanes' private DOMs,
  which the move dissolves, and the driver is hand-rolled rather than a
  mesquite lane, so re-pointing it is arguably M6 arriving early.
  (4) Separately and already true at HEAD, the golden played trace
  refuses because it records trophic grammar revision 0 against this
  build's 8, which takes the played scenario and all three camera arms
  with it and leaves one scripted headed acceptance. Scale if it
  proceeds: about 2,100 lines deleted against 2,400 rewritten, with the
  hard parts already solved elsewhere (the bench proves the scene
  producer, and meristem composes the six per-state roots into one).
- **2026-09-15, Mark ruled on the two M4 questions.** Mesocosm's main
  binary is deferred to a later round rather than migrated with a capture
  rebase; M4 is therefore the assembly, Paredros and Isometry. The stale
  golden played trace is recorded as a finding and left, in the played
  slice plan's findings where that receipt lives, rather than re-recorded
  now.
- **2026-09-15, M5 landed.** Paredros's netrender-drawn `body_sheet` is
  gone: 3,020 lines across nine module files and a binary, net -2,610
  after what was re-homed. Its GUI half, the list, selection, focus,
  scroll, keyboard navigation, buttons, hit testing, two text wrappers
  and the panels themselves, is what the session host has done in
  Cambium since P2, so it was deleted rather than moved. Its product
  half survived in three pieces: the sheet projection was already
  written twice, so the session's panels keep the surviving copy as a
  free `subject_sheet` over the game state, a subject and a selected
  part, with the four projection claims now tested against the session's
  own world; the six world-rule claims stay at their owner in
  `eponym-world`'s equipment tests, where they always were; and
  `equipment_store` moved up to the client crate unchanged, its
  roundtrip test rebuilt over the session fixture. The timed-action
  scene and one of the two parley loaders moved to that binary's own
  hud, its only remaining consumer. §2.2's schematic slot is **not**
  filled and will not be by this milestone: the schematic is stroked
  paths in a netrender scene and the session is a document, so carrying
  it is a rewrite in another medium. It is recorded as the one
  capability the retirement costs, beside the two-process persistence
  receipt, whose unit-tier claim is kept. The rationale lives in
  Paredros's genet document host plan, which already carried the
  retirement as an open decision; that repo has no home for retired
  code. Receipts: 41 isomere tests and 49 with the host feature;
  Paredros check clean but for a pre-existing dead-code warning behind a
  proof feature, 23 lib tests (45 less the 24 body-sheet tests, plus the
  2 store tests kept), 4 new session-bin tests, 6 timed-action tests,
  acceptance ok and the failure scenario exit 1, with 6 of 6 captures
  matching the M4 set to within the two recorded flickers and every
  receipt field identical. Newly recorded: on 1 run in 20 the final
  capture came back with the document scrolled 96 logical pixels,
  content identical; 19 M5 runs and 17 runs of the pre-change binary, 4
  of them under load, did not reproduce it. A scroll-position race at
  capture time, not a layout change. **Open for Mark:** `equipment_store`
  now has no caller. Its discipline is an immutable series of published
  saves and the session writes one mutable save, so which one a Paredros
  save is remains a ruling; both stand until it is made.
- **2026-09-15, M4 landed in full.** Isometry's host is the shared
  assembly plus an 80-line product impl. The hand-written hooks function
  is gone; what is left in `hooks.rs` is the frame tick, the pane, the
  board's pixel grid, the atlas motion, the overmap leaf, the scene
  board's snapshot and the beat hold, with producer registration and
  capture arming on either side of it in the assembly. Isometry's
  `capture.rs` was the source of `isomere::host::Capture` in the
  Paredros half and is now deleted and consumed back under the same two
  environment variables and the same file name, which two tests guard
  because every headed receipt ever taken names them; the PNG dependency
  left the manifest with it. B2's scene leaf is the product's viewport
  key and producer rather than a registration inside the board's sync,
  and `NoProducer` went unused as a result. One addition to isomere:
  `Product::PUBLISHES_ERROR`, default true, because Isometry's DOM has
  no error element and publishing into nothing would have rebuilt the
  retained tree once a frame for as long as an error stood. `main.rs`
  was at the 600-line ceiling, so `boot.rs` took the seam the file
  already had, everything that runs before a window exists, leaving
  main at 432 and hooks at 547. Three behaviour changes, all inert:
  registration moved ahead of the frame tick, a close request now asks
  for one redraw as the window closes, and capture arming runs after the
  beats rather than before. Receipts: workspace check clean with the
  baseline's four pre-existing warnings and nothing new; 372 tests
  before and after, host routing 10, zoom 10 and watchtower 11
  unchanged, the two capture tests replaced one for one; 41 isomere
  tests and 50 with the host feature; Paredros re-verified under the
  changed isomere. Headed captures four times a side in both arms, DOM
  and scene board: final captures byte-identical to baseline runs, with
  the only variation one pixel at (926, 843) that takes both values
  inside the baseline set too. Two corrections for §3: the winit host
  does not re-export `RootView`, so isomere names rootstock as well and
  a one-line upstream re-export would retire that; and isomere's
  manifest comment claiming the root pinned a different mere revision
  was wrong and is fixed. Also found: `cargo check` at the Paredros
  workspace root checks only the two-line root package and never
  touches the client, so the receipt list must name the client
  explicitly.

- **2026-10-01:** the mode host's assessment found no game running on the
  sim (only `mesocosm-genet`'s bench depends on `isocosm`), the three
  overlays stopped at their contract modules, and Eponym's renderling still
  patched to paths outside the repository. Mark ruled the host's location,
  switching and save (442 to 444), carried into §2.4. Design only.
- **2026-10-01:** the knowledge contract ruled (445): a window's access
  follows its seat and subject, never its mode. §2.4 has no open item left;
  the host waits on Mesocosm's M3.
