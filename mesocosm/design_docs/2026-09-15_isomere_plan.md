# isomere: the wing's GUI layer

**Date:** 2026-09-15

**Status, 2026-10-18:** M0 to M5 are landed (the shared sheet, viewport card, examiner, journal, status and help lines, and host assembly); M6 and Mesocosm's main-binary migration stay deferred. The mode host of §2.4 is implemented in `host/` over one native session, with borrowed VTT, Eponym and Mesocosm inputs, transactional saves and per-window attention (case 815), and Pin and Unpin controls are placed in source. Isomere and the combined host pass the all-target, all-feature compiler checks. Open: host runtime and headed two-window acceptance, the host's own gates (switching preserves world, subject and embodiment; knowledge follows the seat) and opaque VTT product-game actuation. Next: verify once the remaining migration pieces are in place.

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

### 2.4 One host and independent presentation choices

**Design refinement, 2026-09-28; not implemented by this document.** Ruling
381 makes each product a build profile of one host over one world save.
The host should distinguish the active play mode and its controls, the
participant's knowledge access, the camera, and presentation detail. The
shared default is a 30-degree 2:1 dimetric view with quarter turns (382,
387), with Mesocosm's opening view still subject to CP1's comparison (388).
Other perspectives remain possible.

Mode changes should preserve the accepted world, subject and embodiment
unless an explicit game action changes them. Camera changes act on the
scene; they do not grant perception or independently select simulation
fidelity. Share picking, highlighting, body inspection and effect inspection
across modes wherever their meanings agree. A ruleset supplies its controls
and readings without owning another copy of the scene's world truth.

The construction examiner should connect source parts and relationships to
capabilities, repertoire, proficiency and conditions. The process examiner
should distinguish supporting requirements, cost contributions, accepted
effects, causal history and the actor's beliefs. Incarnation controls should
support granular configured or randomized carryover (406), with consequences
explained before commitment; the body contract owns the transition model.

**Still open:** the executable's location, build-profile packaging, session
and mode-switch lifecycle, and a shared presentation/knowledge contract.
Isometer currently exposes an orthographic slab camera; perspective support
is future scene work. These are host and scene design tasks, not a request
to move simulation rules into isomere. The above defines the target for W3
and W4 without opening another implementation lane.

**Ruled 2026-10-01 (wing design record, 442 to 444).** The executable lives
in a host workspace of its own, holding only the mode host's binary and
taking the three products' crates by path under the union of their patches;
each product's workspace keeps building its one-mode profile, and Eponym's
renderling, whose patches point outside the repository, retires before
Eponym's mode joins that build (442). A window plays one mode over the open
world and switches in place, swapping controls, view defaults and attention
set while keeping the world, the subject and its embodiment; two modes over
one world on one machine are two windows (443). The world save holds the
sim's history and a section per mode, a build without a mode carrying its
section through unread and unchanged (444). *Reading, not ruled:* the host
can carry no mode until an overlay runs its game on the sim, so it lands
with Mesocosm's M3, carrying Mesocosm's mode alone, and the VTT's and
Eponym's modes join as their overlays land (194, 231). The shared
presentation and knowledge contract is ruled too (445): a window's access
follows its participant's seat (player, creative observer, DM), the entity
it plays and any awareness granted in play (384), never its mode, and every
presentation path, readings, far marks, examiners and the event stream,
reads through it. *Reading, not ruled:* the access type sits in
`isocosm-overlay` beside the attention set, and under replay it shapes play,
not secrecy.

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

- **2026-10-11, live pin connection placed:** `host/src/ui/model.rs` and
  `host/src/ui/view.rs` connect Cambium controls to the existing logged native
  window-attention command. Choices use the current seat-filtered native cut;
  ambiguous event handles are withheld. Local pin state comes from that
  window's binding, not the participant union. Accepted changes refresh all
  windows and retain existing mode-memory/save behavior. Source only under
  826: no compiler run, test, fixture adjustment, tuning or headed receipt.

- **2026-10-11, native attention/host core prepared:** CORE31 is
  applied in the primary checkout, including the native union of active
  windows under 815 and the host command/admission/lifecycle source.
  Source review is qualified; compilation, saved lifecycle and headed
  product binding are still open. The initial execution freeze stopped
  on unrelated compiler ownership. Ruling 824 conditionally admits one
  queue Cargo job alongside unrelated builds after RAM, CPU, GPU and
  SSD checks with separate output custody. The revised guard is being
  reviewed. This does not certify M3 or replace native/spatial/headed
  acceptance. Earlier dated records below are retained.

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

- **2026-10-11, combined migration execution:** CORE31 and the reviewed journey/body/VTT migrations are applied together. Final-state compiler and product checks replace repeated intermediate-state certification. Native attention and shared-world/window lifecycle checks remain required; inventory listing alone does not certify M3.
