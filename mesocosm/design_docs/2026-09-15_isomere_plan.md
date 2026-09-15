# isomere: the wing's GUI layer

**Date:** 2026-09-15

**Status:** design, for Mark's sign-off. No crate founded, no commit.

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
paredros-client, isometry-views and isometry-genet, against mere's cambium
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
  shared assembly plus a product impl, and Mesocosm's main binary moves off
  `chrome.rs` onto it with its five adapters deleted and its headed receipts
  unchanged.
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
   without blocking M0 to M3.
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
