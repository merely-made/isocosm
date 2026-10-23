# The epoch boundary: what moved out on 2026-10-18

**Why this moved.** On 2026-10-18 Mark made the old numbered rulings cases, kept as precedent, with new rules numbered by him alone, and asked every active doc to lose its stacked status banners, old progress entries, stale counters and pasted copies. This file holds, verbatim, what left the live [epoch boundary plan](../../2026-08-01_epoch_boundary_plan.md): its four stacked status banners, the Current state section that sat between section 8 and the Findings (the paragraph carried from the phenotype plan stayed), and the Progress entries other than the four that describe the open work.

## Status banners

**Status, 2026-10-10, after Q3's checkpoint:** native boundary, review,
revision and speciation are implemented; 797/806's metabolic order is
integrated at `97684b89` with focused tests and the native compile gate.
Reproduction and broader invariant/headed certification remain open.

**Status, 2026-10-10:** native boundary, review, revision and speciation
are implemented. Certification is the after-pass's; 797's descending
metabolic-complexity order is still a compliance gate in this lane snapshot.

**Status, 2026-09-26:** Rewritten to the record 2026-09-26 (ruling 280); partially built. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status: partially built, refreshed 2026-09-02.** The world record, causal
event log, species tree, places, reckoning, and player speciation are built,
and since PE3a **the boundary itself happens**: a versioned epoch rule ends it,
every unplayed line takes a scored turn at it, and a driver holds the world
there. Since PE3b the played line's own turn is a screen — the reckoning, a
priced and previewed candidate table, and a committed revision. Goals, NPC
speciation and life-stage plasticity remain design or unwired. The 2026-08-31
reproduction direction is recorded in section 0; earlier rulings are Mark's from the dialogue through 2026-08-03 and
are marked where the reasoning is mine.

## Current state, 2026-10-10

Added under ruling 793; the dated sections above keep their words.

The native owners are `shared/isocosm/src/lineage/boundary.rs`, `review.rs`,
`revise.rs`, `speciate.rs` and `chronicle.rs`, all in that lineage source
directory. `shared/isocosm/src/directing/interim/` holds at the boundary;
`mesocosm/crates/mesocosm-runtime/src/review.rs` and the views present the
played line's offers. The older `mesocosm-core::species`, `score`,
`record`, `history` and `places` owners below are deleted.

At the source snapshot checked for Q1 (`3523420b`), `boundary::adapt`
sorts by living-member count. Ruling 797 requires descending metabolic
complexity defined from catalogue functions expressed plus systems
realized. The reproduction/turn-order lane owns that correction; this doc
pass does not certify its behavior. Significance remains the hagiograph's,
the native session owns history/replay, and the sim's lineage model owns
speciation. The directing plan owns M3 and the interim M4 (800); the full
M4 and contested spatial/resource conditions remain the overlay's gates.

## Progress entries

- **2026-10-10, Q1:** current state/status and native source paths verified
  under 793 to 802; dated receipts preserved. This documentation pass adds
  no compile, test, draw or headed certification.


- **2026-09-02 (PE3b):** §3's reckoning becomes the review's evidence, on a
  screen. `World::offers` puts every candidate the played line could commit on
  one table with the score it earned, the price its next descendant would pay,
  a founder preview and the reason it cannot be taken when it cannot; the
  driver's `Review` adds the epoch's readings — each carrying whether it took
  the record, which is abnormality-against-the-record made legible — and the
  bounded ecology windows. §8 q4's flat price is honoured unchanged: nothing is
  spent at the commit, and the finite thing the table is weighed against is
  **the founder's own material**, the ecology's own provisioning arithmetic
  rather than a currency. §2's *adopt or branch* is still one player and one
  child revision; nothing here merges a proposal, and §8 q6 (who must adopt a
  shared continuation) is untouched.

- **2026-09-02, terminology note (doc only):** Mark authorized renaming the
  memorial organ to **hagiograph** and reassigning **tulpa** to gemot's
  federated adapter-training lane. A dated note was added at this doc's
  tulpa mention rather than rewriting the historical text; see repo
  `CLAUDE.md`. No code changed.

- **2026-09-02:** the trait array's deletion ruled; the ruling lives in the
  [phenotype plan](../2026-10-10/2026-07-31_phenotype_plan.md)'s progress, and this is the
  module the entry below already found had no consumer left in `World`.
- **2026-09-02 (PE3a):** the boundary finally happens on its own. `EpochRule`
  is a versioned world rule beside the ruleset — `Timed { ticks }` built,
  default 1,000, `Gated` and `PlayerTriggered` named as data that end nothing —
  and `World::apply` ends the epoch when the budget is spent, because a
  headless enclosure has to obey the same rule a driven one does. **The
  reckoning stayed a separate call** (`World::reckon`) for §12's own reason: it
  reads the past, and the past lives beside a world. The driver does that half
  and a replay does it through the same function, which is what keeps a run
  through a boundary replaying to the same hash. `Runtime::end_epoch` remains
  the manual door for ending one early.

  At the boundary every unplayed line takes a turn in `World::adapt_round`, in
  §6's own initiative order — descending recipe complexity, ties by id, commits
  landing immediately so a later line answers a world the earlier ones changed.
  The **ordering idea from `epoch.rs` was kept and its trait array and
  `fitness` were not**: a candidate is scored by growing it in a copy of the
  world and reading the flow record, income against rent, which is Mark's
  ruling of 2026-09-01. Nothing authors a significance table, and the
  storyteller still picks no outcomes.

  §8 q4 is honoured as ruled: a revision is priced flat, and the descendant
  pays only the ordinary development price. §8 q2 (does an unplayed lineage's
  speciation get a name) is untouched — a round commits revisions, never a
  split.

- **2026-08-31:** separated reproduction's individual checkpoint from this
  lineage checkpoint and linked the playable ecology integration plan. No
  implementation or earlier boundary ruling changed.

- **2026-08-01:** speciation-by-act, significance-as-abnormality, the scoring
  axes, the reward ladder, goal selection, and life-stage plasticity recorded
  from dialogue. No implementation added.
- **2026-08-01:** recorded shared-lineage authorship: every adaptation produces
  an immutable child revision; agreement shares a continuation and disagreement
  branches without forcing every revision to become a species.
- **2026-08-03:** resolved the co-signing target: co-players adopt one
  developmental program, while world-conditioned phenotype realization remains
  free to vary.
