# Docs dedup plan

**Status, 2026-10-06:** done; D0 to D5 landed, the residue ruled (616 to 619).

Carries out the wing design record's ruling 593 ("each result one home, other
mentions a link") as Mark shaped it in rulings 612 to 615. The record holds
the rulings; this plan holds the work.

## Baseline (2026-10-06, isometry `6fbeb889`)

Measured with ten-word runs shared between and within the tracked `.md`
files (scripts kept with the session; the method is stated so it can be
re-run).

- 133 docs: 107 live, 61,411 lines; 26 archived, 16,661 lines.
- The wing design record: 160,016 words. §0's rulings are 93,475 words over
  8,306 lines; its Progress log is 19,506 words over 1,679 lines, mostly
  one-line restatements of those rulings.
- The wing index (`mesocosm/design_docs/DOC_README.md`): rows are 11,819 of
  its 13,367 words, the record's row alone 2,737. The root index's rows
  reach 370 words, Eponym's 303.
- `DOC_POLICY.md` three times (root, Mesocosm, Eponym), one canonical core
  (lines 1 to 123) with stale addenda ("Paredros"; Isometry as "a single
  subsystem").
- The product `CLAUDE.md` files share about 400 runs; the sim design session
  notes share about 600 with the record, mostly round tables.
- Five broken relative links in live docs: four into plans archived without
  their links repaired, one into a crate the migration moved.

## Phases

Each phase lands as its own commit on main, docs only.

### D0. Links

Repair the five broken live links.

*Done when* the link check finds no broken relative link in a live doc.

### D1. Indexes (ruling 612)

Every row of the three `DOC_README.md` indexes becomes the link, one line of
what the doc is for, and a dated status. Ruling summaries leave the rows.

*Done when* no index row restates rulings, no row exceeds 80 words, the wing
index is under 3,000 words, and every live doc in each `design_docs/` is
indexed exactly once.

### D2. Progress logs (ruling 613)

The record's Progress log and every live plan's are cut to landings: what
was built or verified, with its commit where one is known, rulings named by
number only. An entry that only restates a ruling goes.

*Done when* no progress entry restates a ruling's content, and the record's
log is a list of landings.

### D3. Session notes and round tables (ruling 614)

Extract what is still open in the sim design session notes into a live doc,
then move the notes to `mesocosm/design_docs/archive_docs/2026-10-06/` and
repair links to them. Briefs that tabulate rounds (the anatomy brief first)
keep their design text and cite rulings by number.

*Done when* the notes are archived with every open item carried into a live
doc, no live link points at the old path, and no live brief re-tabulates a
round's questions and answers.

### D4. Copies (ruling 615)

One `DOC_POLICY.md` at the root, its addendum covering the three products;
the product copies become a link to it. The rules the product `CLAUDE.md`
files share move to the root `CLAUDE.md`, and the product files link there.
`LICENSES.md` files stay.

*Done when* the product `DOC_POLICY.md` files hold only a link, the root
copy's core is byte-identical to the canonical core, and no rule appears in
both a product `CLAUDE.md` and the root one.

### D5. What remains

Re-run the measure and list what still repeats (the sim plan's schema tables
beside the rulings they carry, the overlay plans' shared opening, the paging
receipts). Anything further to cut goes to Mark as a fork; dated rulings keep
their words.

*Done when* the residual list is in this plan's Findings and its forks are
put.

## Findings

- 2026-10-06, D5: after D0 to D4 the repeated-paragraph surplus fell from
  3,192 words to 1,168. What still repeats: the record against the sim plan
  (827 ten-word runs) and the anatomy brief (491), a spec sharing what its
  rulings set, kept by ruling 616; the overlay plans' copies of §11, linked
  under 617; twenty stale status lines, brought current under 618; fifteen
  broken links in older archives, repaired under 619; the paging receipts'
  summary and final-cost pages (629 runs), receipts kept as taken; and the
  two `LICENSES.md`, per-product legal records kept by 615.
- 2026-10-06, D4: the root's `DOC_POLICY.md` core is byte-identical to
  genet's, the canonical core; mere's copy differs at line 80 (bold where the
  core has backticks), a diff for mere's own lane. The product `CLAUDE.md`
  files' pipeline-law lines disagreed: Eponym's lacked the 2026-09-24
  materials amendment, so the one root copy takes Mesocosm's amended wording.
  The two rollback-netcode rules stay with their products, their substance
  differing (Eponym's puts single-player first and asks for a conflict UI).
- 2026-10-06, D3: every item the session notes left open (§6, §8.5, §9.5,
  §10) was found ruled since or held in a live doc: the parked questions in
  the sim plan's §8, the naming items by rulings 157, 224, 251 and 252 with
  `borg` to construct in the record's §3.2.1, Massif Press in ruling 108 and
  the sim plan, the mode host by rulings 442 to 445, the threats by 283, the
  overlays' forks by 232 to 250, the far rungs' lens by 434, and examining on
  approach in the record as not ruled. Nothing needed extracting. Only the
  notes tabulated rounds; the anatomy brief narrated its rounds in its log.
- 2026-10-06, D2: some plans hold rulings Mark made before the design record
  existed (the board plan's cliff height and DOM board, the dev tools plan's
  fixture defaults, the isomere plan's M4 answers, the phenotype plan's trait
  array). The record has none of them, so their plan is their one home and
  they stay where they are; only the trait array's second copy, in the epoch
  boundary plan, became a link.
- 2026-10-06, D1: many docs' own status lines are older than the status their
  index row carried (the sim plan's still says implementation in progress on
  2026-09-22). The rows now carry the newest status; bringing each doc's own
  line up to date is left for D5's list.

## Progress

- 2026-10-06: D0 done. The five broken live links repaired: four into
  archived plans, one into the save-growth probe the migration moved.
- 2026-10-06: D1 done (`44de9664` and this commit). The three indexes are
  rows of link, purpose and dated status: wing 13,367 words to 1,911, root
  2,772 to 979, Eponym 2,815 to 536; longest row 39 words; every live doc
  indexed once. The prose they carried was checked against the docs first:
  every commit, count and date found a home, and the one that had none moved
  to the epoch boundary plan.
- 2026-10-06: D2 done. The design record's log cut from 1,676 lines to the
  eight landings no other doc owns (`04a7eb45`), the sim plan's from 415 to
  its fifteen landings, both archived intact at
  `mesocosm/design_docs/archive_docs/2026-10-06/`; the three overlay plans,
  the rename, place-graph and epoch boundary plans' ruling restatements cut to
  their landings.
- 2026-10-06: D3 done. The session notes archived at
  `mesocosm/design_docs/archive_docs/2026-10-06/`, links repointed in and
  out; the anatomy brief's round-by-round log cut to its two landings.
- 2026-10-06: D4 done. One `DOC_POLICY.md` at `design_docs/`, its addendum
  covering the three products, the product copies a link to it; the rules
  the product `CLAUDE.md` files shared, with the root's own copies, gathered
  once in the root `CLAUDE.md` under "Rules for all three products", the
  product files linking there.
- 2026-10-06: D5 done. The residue measured and put to Mark as rulings 616
  to 619, each carried out: overlay plans linked to §11 (`04eb72f4`), status
  lines current, archive links repaired (`4f2ea2ca`).
