# Docs dedup plan

**Status, 2026-10-06:** in progress.

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

## Progress
