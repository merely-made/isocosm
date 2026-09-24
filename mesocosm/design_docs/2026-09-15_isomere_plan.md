# isomere: the wing's GUI layer

**Date:** 2026-09-15

**Status:** design, for Mark's sign-off. No crate founded, no commit.

**W1, 2026-09-18:** keep. Tier: stack, hosting and interface. Rewrite only
the keymap row per ruling 21. Evaluated against the wing design record; see
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md)
§2.

**Ruling it serves (Mark, 2026-09-14):** isometer is the game-world scene
family; **isomere** is the wing-unique GUI layer, scene-plus-host chrome,
overlays, graphs and the applications of Cambium that belong to the wing
rather than to one product. isometer never names a Cambium widget; isomere
never names a voxel. Mark's steer of 2026-09-15: lift the existing game GUIs
and evolve them with Cambium, rather than design a fresh one.

**Owns:** what the three products have each built over Cambium and netrender
that is not product rules, promoted once and consumed back; the wing's
shared stylesheet tokens; the seam between a product's vocabulary and the
shared shapes. Lives at `shared/isomere` beside isometer, its own workspace,
MPL-2.0, `publish = false`, path-consumed by all three products, pinned to
the same mere and genet revisions as the family.

**Does not own:** the scene (isometer), the scenario lane (mesquite),
widgets that belong in Cambium itself (those go upstream to mere), any
product's rules or vocabulary, or the products' hand-rolled netrender HUDs
except as the thing they migrate off.

---

## 1. What the inventory found

Read-only survey of 2026-09-15 over mesocosm-genet, mesocosm-views,
eponym-client, isometry-views and isometry-genet, against mere's cambium
family at 876320fd.

**The same panels exist three times under three names.**

| Shape | Mesocosm | Paredros | Isometry |
| --- | --- | --- | --- |
| Viewport card: a `custom_leaf` with `role="img"` and a label in a `.scene-card` | bench `view.rs` | session `view.rs` | board pane; overmap leaf |
| Part or tile examiner | `dev/part.rs`, `app/inspection.rs`, `section/inspection.rs` | session `panels.rs`, `body_sheet/schematic.rs`, `details.rs` | `sheet.rs`, `board/tokens.rs` |
| Journal or message log | `review.rs` rows | session `.glyphs`, `#journal-summary` | `panel.rs` messages |
| Status or vitals panel | `vitals.rs`, `succession.rs` | session `.status-lines`, timed_action lines | `.side-status`, `state/play.rs` |
| Capture and receipt lane | `app/receipts.rs`, `played.rs` | session `probe.rs`, `smoke.rs` | `capture.rs`, `selftest/*` |
| Scenario adapter over mesquite | `bench/probe*.rs` (662 lines) | session `probe*.rs` (562 lines) | none; taproot selectors direct |
| Error line, control-help line | `#notice`, `.body-menu-keys` | `#viewport-error`, `#controls-help` | `.side-status`, `.side-hint` |
| Dev overlay | `dev.rs` in a workbench tile | `frame_health.rs` | env-var self-tests |
| Comparison strip | `bench/comparison_view.rs` | body sheet life comparison | none |

**Three stylesheets are one sheet in three palettes.** The Mesocosm bench
sheet and the Paredros session sheet are rule-for-rule the same in the same
order (reset, root, header, headings, `main` flex, `.scene-card`,
`.viewport`, `.toolbar`, `button` with its focus outline, `.parts`,
`.part.selected`, `.reading`, `.field`, `.field-name`, `.field-value`);
only six colours, four font sizes and three box numbers differ. Mesocosm's
six view sheets repeat one token set and restate Cambium's own
`.detail-*` rules four times. No product derives its palette with tinct;
all three are hand-picked hex.

**Product code reimplements Cambium.** Mesocosm's `chrome.rs` plus five
per-panel adapters (about 1,800 lines) are one 150-line arrangement
written five times and duplicate what `cambium::surface` and
`cambium-genet-winit-host` do once for the other two products; its `dev.rs`
hand-walks a workbench `TileTree` and carries a stale note that no genet
surface renders one, which `cambium::frisket` now does. Paredros's
`body_sheet` (2,400 lines) is a second GUI toolkit drawn straight into a
netrender scene: list, selection, focus, scroll, keyboard navigation,
buttons, hit testing and a parley font loader that appears twice. Each
bench and session view defines a private `button()` and `field()`.
Isometry's `overlay_panel` is a private titled panel; its `atlas_labels.rs`
is product-neutral font-to-callout machinery in a product host.

**Isometry is the healthiest consumer** (Cambium throughout, seven host
hooks, the only stated stacking discipline) and Mesocosm's main binary is
the only one still on a hand-rolled winit host.

## 2. The shape of isomere

Three tiers, each promoted from an existing product implementation, never
written fresh, and each consumed back by the product it came from in the
same lane that promotes it.

### 2.1 Tokens and sheet

One `isomere::sheet` producing the shared rules the three sheets already
agree on, parameterised by a `tinct`-derived palette: reset, root layout,
headings, the viewport card, toolbar and button states, parts palette,
reading column and field rows, journal rows, status lines, the error and
help lines, and Cambium's `.detail-*` vocabulary stated once. A product
supplies seeds (its two or three hex anchors) and its own additional rules;
isomere derives the rest with `tinct::derive_palette` and `best_on` so
contrast holds in both themes. The class vocabulary is the one already in
use across the bench and session sheets, so promotion is a move, not a
rename.

### 2.2 Panels

Cambium view functions over product-neutral inputs, promoted in this order
because each has three sources to reconcile:

1. **Viewport card**: leaf key, size, label, optional overlay children,
   the error line beneath. From the bench and session `viewport()`.
2. **Examiner**: a list of addressable parts with selection, a reading
   column of name-value rows, optional schematic slot. Inputs are a small
   `ExaminerModel` (rows with an id, label, state class, selected flag;
   readings as name-value pairs) that each product builds from its sheet
   projection. From Paredros's session panels and `body_sheet`, Mesocosm's
   `dev/part.rs`, Isometry's `sheet.rs`.
3. **Journal**: ordered rows with a mark, a headline, a founding line and an
   optional live line, plus a summary line. From Paredros's glyph journal
   and Isometry's message log, with Mesocosm's review rows as the third
   shape.
4. **Status panel** and **control-help line**: lines with optional
   emphasis; the help line from a declared keymap so it cannot drift from
   the handlers.
5. **Comparison strip**: N viewport cards with a chosen flag. From the
   bench.
6. **Capture and receipt panel** and **dev tile**: the receipt-on-exit
   lane already in mesquite, given one panel that shows the last capture
   path, the frame profile and the notice; the dev tile is a frisket pane
   over a workbench tree, replacing Mesocosm's hand-walk.

Every panel takes its model by value or reference and emits Cambium
elements with the shared classes; no product type crosses into isomere.

### 2.3 Host assembly

One `isomere::host` over `cambium-genet-winit-host`: the seven hook
closures with the common parts filled (producer registration for viewport
leaves, the mesquite lane wired into `after_frame` and `close_request`,
capture arming, the error line published from producer errors, the frame
profile line), leaving a product a `Product` trait of the same spirit as
mesquite's: its state, its root view, its sheet seeds, its keymap and its
actions. Isometry's `hooks.rs` and Paredros's session `model.rs` are the
two sources; Mesocosm's main binary is the third consumer and the one that
retires `chrome.rs` and its five adapters by moving onto it.

## 3. What goes upstream instead

Anything that is not wing-specific goes to mere's cambium family, not to
isomere: the private `button()` and `field()` helpers (Cambium has `button`
and `setting_row`), the titled overlay panel if `overlay_surface` plus
`summary_body` do not already cover it, the two parley font loaders (the
host owns a text system), and `atlas_labels.rs` if graphshell's atlas
callouts want it. isomere's rule for the boundary: if a second non-game
consumer could want it, it is Cambium's.

## 4. Lanes and done-conditions

Each lane promotes one tier from its sources and retargets those sources in
the same commit, gated on every product's existing receipts: Mesocosm's
bench acceptance, Paredros's session acceptance with its receipt hash, and
Isometry's host routing, zoom and watchtower suites.

- **M0 found.** `shared/isomere` with `sheet` and the palette derivation;
  the bench and session sheets become seed sets plus product rules. Done
  when both products render byte-identically to their current captures with
  the same seeds (the derived palette must reproduce the hand-picked hex
  for the seeds given, or the capture set is rebased with rationale).
- **M1 viewport card and error line.** Done when the bench, the session and
  the overmap leaf use it and their scenarios pass unchanged.
- **M2 examiner.** Done when Paredros's session sheet panel and Mesocosm's
  parts examiner are the shared panel over their own models, `body_sheet`'s
  schematic is the optional slot, and the P3b and Bench B scenarios pass.
- **M3 journal, status and help.** Done when the glyph journal, the message
  log and the review rows are the shared journal; the help line is derived
  from each product's keymap; scenarios pass.
- **M4 host assembly.** Done when Isometry's and Paredros's hosts are the
  shared assembly plus a product impl. Mesocosm's main binary was in this
  lane and is deferred to a later round by Mark's ruling of 2026-09-15:
  its chrome lays out in physical pixels against the shared host's
  logical ones, so the move rebases its capture set by construction,
  which is not a cost this round is carrying. When it does run, it moves
  off `chrome.rs` with its five adapters deleted, and it answers the two
  questions the assessment left: what replaces the played receipt's
  `frame_graph` block behind a producer, and whether the scenario
  grammar's `assert text` becomes a host-DOM assertion with the driver
  moved onto a mesquite lane.
- **M5 body sheet retirement.** Done when Paredros's netrender-drawn
  `body_sheet` is archived with rationale and its equipment session's
  claims live in the session host's examiner; 2,400 lines gone.
- **M6 dev tile and receipts panel.** Done when Mesocosm's dev tile is a
  frisket pane and all three hosts show the same receipts panel from
  mesquite's lane.

Order: M0 and M1 first (small, every product touched once), then M2 and M3
in parallel (disjoint panels), then M4, then M5 and M6.

## 5. Decisions for Mark

1. **Palette derivation versus preservation.** Deriving with tinct changes
   pixels unless the seeds reproduce today's hex; the cheap path is to seed
   from today's exact colours and let tinct fill only the states nobody hand
   picked (hover, focus, disabled). Recommended.
2. **Mesocosm's host migration is in scope.** M4 moves the one remaining
   hand-rolled winit host; that is the largest single change and the one
   with the most headed receipts. It can be deferred to a later round
   without blocking M0 to M3. **Ruled 2026-09-15: deferred**, on the
   measurement that the physical-versus-logical pixel difference rebases
   its captures however the move is done. The assembly it would have
   moved onto is built and proven by Paredros, so the deferral costs the
   later round nothing but the product work itself.
3. **Naming inside isomere.** Panel names above are plain working words
   (examiner, journal, status); nothing is coined.

## 6. Findings

- The three products' probe adapters are the precedent: mesquite took the
  lane, each product kept about 400 lines of "what is about this host".
  isomere's panels should leave a similar residue, not zero.
- isomere depends on isometer for nothing; a viewport card holds a leaf key,
  not a scene. That is the boundary the ruling drew and the inventory
  confirms it holds.

## Progress

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
- **2026-09-16, Mark deferred all of M6 with the migration.** M6 is the
  dev tile as a frisket pane and one receipts panel shown in all three
  hosts. Both halves touch Mesocosm's main binary, whose host migration
  was deferred on 2026-09-15: its dev tile is drawn by the chrome, so it
  cannot become a document pane without that move, and a receipts panel
  in "all three hosts" with the bench standing in for the main binary
  would not be the lane as written. So M6 waits whole for the round
  that moves Mesocosm's main binary, and lands the panel in all three
  real hosts at once. isomere is therefore closed at M5 for this round:
  the shared sheet, the viewport card, the examiner, the journal, the
  status and help lines with a declared keymap, and the host assembly,
  consumed by Paredros and Isometry, with Paredros's netrender body sheet
  retired.
