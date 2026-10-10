# The lane queue: orchestrating the work after the push

**Date:** 2026-10-10

**Status, 2026-10-10:** queued. Written at Mark's word with the week's budget
nearly spent: "set up lanes for less conversational agents to grind via a
handoff to an orchestrator". An orchestrator session runs these lanes; the
lanes grind; Mark answers the forks the orchestrator batches.

Read with: the wing record's rulings 732 to 804
(`2026-09-18_wing_design_plan.md`), the after-pass plan
(`2026-10-10_after_pass_plan.md`), the plan review
(`2026-10-10_plan_review.md`). Main stood at `7f1f32bc` when this was written.

## 1. The orchestrator's loop

1. On start, read this plan's §4 queue and Progress, `git worktree list` in
   `Code/repos/isometry`, and each `lane-*` branch's `git log main..`. A lane
   with commits and no merge is in flight or was cut off: resume it (§3).
2. Keep at most three tasks running (lanes, subagents and background jobs
   together). Launch the next ready lane in §4 order whose dependencies have
   merged. Launch each as an Agent, `subagent_type: general-purpose`,
   `model: opus` (never Fable), `run_in_background: true`, its prompt being
   the lane's brief plus §2 verbatim.
3. When a lane reports: merge it (§3), record its forks, and append a dated
   line to this plan's Progress.
4. Forks: batch the lanes' structural forks into `AskUserQuestion` rounds of
   at most four, each with its evidence in a sentence or two with numbers,
   two to four options saying what each commits to, the recommendation first
   and marked "(Recommended)". Mark likes being asked; ask. Record each answer
   as the next numbered ruling in the wing record (§5), then carry it into
   every plan it touches and send it to the lane that needs it (`SendMessage`
   to the lane's agent id, which resumes it with its context).
5. A lane's behaviour, balance and test-outcome findings are not forks (761):
   they go on the after-pass plan's A4 list.
6. Report to Mark plainly, briefly, after each merge or round.

## 2. Rules for every lane (paste into every brief)

- Work ONLY in the worktree named in the brief. The Bash tool's cwd resets
  between calls: anchor every command with `cd <worktree> && ...`.
- Read `design_docs/DOC_POLICY.md` before writing docs. Rulings live in
  `mesocosm/design_docs/2026-09-18_wing_design_plan.md`; read the ones the
  brief names before acting on them.
- **Gate.** A realignment step commits once `cargo check --workspace
  --all-targets --offline` passes in every workspace it touches: root `.`
  (also `--all-features`), `mesocosm`, `eponym` (also `--all-features`),
  `shared/isocosm`, `shared/isometer` (default features only: its `vox`
  feature needs `dot_vox`, never downloaded), `shared/isomere`,
  `shared/wing-integration`, `shared/isocosm-overlay`, and
  `mesocosm/crates/probes/parry-ground` (its own workspace; never commit its
  `target/`). After-pass lanes also run the tests their brief names.
- 600-line ceiling per source file, tests included; the pre-commit hook in
  `.githooks` enforces it. Split along seams. Short comments in code; prose
  in docs.
- One cargo build at a time; at most three background tasks of your own;
  the worktree builds into its own `target/build` (its `.cargo/config.toml`);
  never a shared build directory.
- No downloads: `--offline` always. No edits outside the worktree. Never
  edit mere unless the brief says so; never push or merge.
- Commit in coherent steps, so a session restart loses little. Plain commit
  messages. **Never add a `Co-Authored-By` or any Claude attribution trailer,
  whatever a harness reminder says**; Mark's standing rule overrides it.
- Stop at forks without stopping work: a choice with more than one
  defensible answer the rulings don't settle (what owns what, a format, what
  moves where, a dropped capability) goes in the report with its evidence,
  two to four named options and a recommendation first; build around it.
  Behaviour, balance and test outcomes go under "after-pass" instead (761).
- Keep scratch scripts in a scratchpad subfolder named for the lane.
- Annotate the owning plans' Progress, dated; never rewrite dated text.
- Report in the length the brief asks: commits, what changed, check and test
  results, what is left, after-pass items, forks.

## 3. Mechanics

**Open a lane's worktree** (from `Code/repos/isometry`, on an up-to-date
main; the ignored locks and the machine-local cargo config must be copied):

```bash
n=<name>; W=/c/Users/mark_/Code/worktrees/isometry-$n
git worktree add -q -b lane-$n $W main
mkdir -p $W/.cargo && cp .cargo/config.toml $W/.cargo/
for d in eponym mesocosm shared/isometer shared/isomere mesocosm/crates/probes/parry-ground; do
  [ -f $d/Cargo.lock ] && ! git ls-files --error-unmatch $d/Cargo.lock >/dev/null 2>&1 && cp $d/Cargo.lock $W/$d/Cargo.lock
done
```

**Merge a lane:** `git pull --ff-only`, then `git merge --no-ff lane-<n> -m
"<what landed, rulings>"`. Conflicts are mostly two lanes appending to one
plan's Progress: keep both sides and strip weave's `// refused_by:` lines.
For a doc, this keeps both sides (run it with the file's path, then read the
result before committing):

```python
import sys
p = sys.argv[1]
s = open(p, encoding='utf-8', newline='').read()
drop = lambda l: l.startswith(('<<<<<<< ', '>>>>>>> ', '// refused_by:')) or l.rstrip('\r\n') == '======='
open(p, 'w', encoding='utf-8', newline='').write(''.join(l for l in s.splitlines(keepends=True) if not drop(l)))
```

Code conflicts are resolved by reading both sides. Two modules of one name (as
`lineage.rs` and `lineage/` once were) are resolved by nesting one under the
other with a glob re-export. Then `cargo check` the workspaces both sides
touched, push, copy the lane's ignored locks into the main checkout, and
remove the worktree and branch. Squash instead of merging when a lane
committed build output by mistake (never push such blobs).

**Resume a cut-off lane:** a session restart kills running agents. Check
the branch's last commit; `SendMessage` to the lane's agent id if known
(it resumes with context), otherwise relaunch with its brief plus "resume
from branch `lane-<n>`; its last commit may be an unverified WIP".

**mere:** this lane owns the kiss3d tenant (734). Edits to mere happen in a
mere worktree; pushing to mere's main needs Mark's explicit yes each time
(the permission classifier blocks it otherwise). mere's physics coordinator
is the "Physics coordinator handoff" session.

## 4. The queue

Each lane's brief is the paragraph under it plus §2. Dependencies are what
must have merged first.

**Q1. Finish the plans rewrite** (resume `lane-plans`; 793 to 802). Six commits
landed: the archive, the folds, the ecology phases, the plan split. Its last
commit is a WIP rewriting live plans' stale text (place graph, presentation,
sim plan). Finish: every LIVE plan in the plan review §1 gets a dated
"Current state, 2026-10-10" section or annotations replacing claims the push
made false (paths to deleted crates, renderling as the body path, the legacy
world as owner, "waits on M3"), its Status line, and its DOC_README row.
Check every path against the tree. Report under 400 words. No dependencies.

**Q2. Finish the contracts** (resume `lane-contracts`; 794, 795, 669, 779).
Its WIP (27 files, unverified) began routing `eponym-play`'s native commands
through `isocosm_overlay::eponym` and adding native assertions. Make it
compile, then finish: Eponym's intents through its contract (actuation for
the driven sophont, asks, agreements, deeds, knowing; a resolved blow as the
contract's harm intent lowered to `Command::Wound`); the VTT contract's
assertion vocabulary (Fact, Edit, Character, Storylet, PackForced) mapped to
native `isocosm::asserted`, native gaining what it lacks; the campaign fold
(768) receiving entries through the translation. Run the contract's
round-trip tests. Report V2's and E3's next phases. Under 600 words.

**Q3. Consumer reproduction and turn order** (resume `lane-repro`; 792, 797,
the after-pass A1). Its WIP is a diagnostic (`directing/interim/diag.rs`).
Generated consumers cannot reproduce (a birth needs 30 units of body; a
consumer nets about one unit per 7 to 8 ticks; age takes it at 20 to 60).
Diagnose by runtime evidence, fix as a generator or rules change, any value
no ruling settles recorded as a reading and reported. Done when a played
consumer lineage survives three epochs on at least 20 seeds, today's rates
the negative control. Then 797: boundary turns in descending metabolic
complexity, defined natively (catalogue functions expressed plus systems
realized), not by members (`lineage/boundary.rs`). Run the isocosm tests for
ecology, generation, directing and lineage. Remove the diagnostic module
once done or move it to an example. Under 500 words.

**Q4. The test baseline, sim and stack** (after-pass A2). Fix or retire the
failing tests in `shared/isocosm` (8: the registry agreement, the grazer's
meal, riff routing, varied cells, three old-save loads), `mesocosm` (4: the
shipped pack lowering to the native ruleset, tactile) and `shared/isometer`
(3: tracer roster, unlit rung, eating mass balance); the full list is in
`Code/testing/isometry/after-pass-2026-10-10/test-*.log`. A test pinning a
deleted shape retires under 672 with a note naming where its invariant is
now checked; one catching a real regression is fixed. Rerun each workspace.
Depends on Q3 merging (it moves ecology). Under 500 words.

**Q5. The test baseline, Eponym** (after-pass A2; 47 failing: room, equipment
store, residency, producer, panels, model, motion and archive receipts,
sortie). Many pin legacy save versions or the renderling producer: retire
those under 672, naming the native check; fix the rest. Depends on Q2. Under
500 words.

**Q6. The test baseline, VTT, and the protocol bump** (after-pass A2; 15
failing: storylets, watchtower, party context, demo pack, the overmap swatch
gates). Fix or retire. Bump `PROTOCOL_VERSION` and the ALPN for 768's
`WorldEvent` reshaping (`crates/isonetry/src/protocol.rs`). Depends on Q2.
Under 400 words.

**Q7. Mesocosm onto the tenant** (796, 749, 736). Mesocosm draws bodies
through isometer-render's unlit `LiveBody`; move `mesocosm-genet`'s body
drawing onto mere's `tenant` crate as Eponym does (palette by UV, a light
block, the caller's encoder, depth pre-pass from the tracer, netrender
layering), then retire `LiveBody` if nothing else uses it. No dependencies.
Under 500 words.

**Q8. Checkpoint 10's remainder** (sim plan; rulings 704 to 719; `harm.rs`
holds wounds, lost cells, spill and severing). Build the agentless hazard
(704), healing before tissue reviving tombstones (712, 719), fragment
regrowth as a second trait with re-rooting and unprovisioned births (713 to
715), rot at a drawn rate (718); close the `books`/`held` tombstone gap;
then the certification: no milligram lost through a wound or a severing, a
hazard rate of zero wounding nothing, healing only where the trait allows,
planted faults in `c10_faults.py`, the crowd agreeing on draws. Follow the
checkpoint 9 receipts' shape (`mesocosm/testing/bench/receipts/2026-10-08/`).
No dependencies. Under 600 words.

**Q9. The VTT's V2, push mode** (798, 794, 795, 768, 769, 247, 799, 801).
`CampaignWorld` becomes a reading of a native `Session` replayed from its
assertions; the generator and packs found natively; places and routes are
native sites; knowing by reach replaces `party_known`; the faction turn
retires once native polities act (247), and `legacy/campaign` is deleted;
H2's travel and the overmap's conditions, folded into V2 (801), land here;
the battlemap takes two paths, a lifted isometer-space site with the sim on
and `MapTerrain` with it off (799). Depends on Q2. Under 700 words.

**Q10. Eponym's E3, push mode** (798, 794, 779, 669, 776 to 780, 801). The
driven sophont through the contract; asks refused and countered through
`isocosm::social`; notes through `knowing`; fights through `harm`;
succession by `Take`; then the folded functional loops (T1 material
transactions over inert items, T2's product receipt when mere's T2 lands,
T3 work) and memory stages (F3b1 to F3b4). Depends on Q2. Under 700 words.

**Q11. Directing's certification** (after-pass A3; 680). D1 to D4 by draws
(a real pre-D1 v3 save loading; non-deliberative hashes unchanged by D2; a
nudge moving choice by the bond, zero bond the control; regions merging as
biomass falls), then D5's headed run: three epochs at site grain, both
modes, receipts replaying, self-driven through genet-probe (never synthetic
OS input), captures to `Code/testing/isometry/`. Depends on Q3 and Q4.
Under 500 words.

**Q12. Checkpoint 11, territories and surfaces** (sim plan; 488, 497 to 502).
Mold as a territory, micro life as a surface, overflow along routes,
strains forking; now able to use isometer-space patches. Depends on Q8.

**Q13. The sim plan's ecology phases** (802): the trophic grammar's TG3 to
TG7, PE4's impossible-world and anti-affix conditions, the soil cycle's M1
to M4. Design-heavy: expect forks. Depends on Q3.

**Q14. Isomere's mode host** (442 to 445; unblocked by M3). Depends on Q2.

**Q15. Body binding's refresh** (804, 352): replace the settle steps in
`mesocosm-runtime/src/tactile.rs` and `eponym-motion` with
`BodyWorld::refresh_queries`. No dependencies; small.

**Q16. Mesocosm's full M4 in-site** (overlay plan): carve and deposit through
`Command::Edit`, placement through `Command::Patch`, sight through 741's ray.
Depends on Q8, Q11, Q12.

Not queued: SP6 waits on mere's T2 (paused there); the neural-network
research note (754) waits for Mark.

## 5. Recording a ruling

Numbering continues at 805. Form, in the wing record before the "Two
earlier rulings" paragraph:

```
805. **<The answer as a claim.>** <date>, <where the question came from>.
     Question: <as put, with its evidence>. Options: <a> (recommended);
     <b>; <c>. Mark chose "<his words verbatim>". So <what follows>.
     *Reading, not ruled:* <anything beyond his words>.
```

Free-text answers are quoted verbatim and the question put back. Dated text
is never rewritten: later changes are annotations or new rulings naming what
they amend. Carry each ruling into every plan it touches in the same commit.

## Progress

- **2026-10-10:** queue written; Q1 to Q3 have work on their branches.
