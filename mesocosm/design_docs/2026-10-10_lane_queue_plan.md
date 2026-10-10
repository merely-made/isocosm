# The lane queue: orchestrating the work after the push

**Date:** 2026-10-10

The newest Progress entry below owns execution state. The following block
records an earlier resumed snapshot; later checkpoints and parked lanes
are recorded in Progress without replacing that history.

**Current state, 2026-10-10:** in progress. Q1 and Q7 are integrated; Q2,
Q3 and Q8 are active. Q3's anatomical accounting and turn-order checkpoint
is integrated, with its reproduction gate still open. Q8 reuses Q7's
finished worktree. Structural forks for character metadata and healing
allocation are pending; the next numbered ruling remains 809.

Original setup record:

**Status, 2026-10-10:** queued. Written at Mark's word with the week's budget
nearly spent: "set up lanes for less conversational agents to grind via a
handoff to an orchestrator". An orchestrator session runs these lanes; the
lanes grind; Mark answers the forks the orchestrator batches.

Read with: the wing record's rulings 732 to 804
(`2026-09-18_wing_design_plan.md`), the after-pass plan
(`2026-10-10_after_pass_plan.md`), the plan review
(`2026-10-10_plan_review.md`). Main stood at `7f1f32bc` when this was written.

## 1. The orchestrator's loop

**Execution update, 2026-10-10 (805):** resumed in Codex from main
`8c8b49bb`. Mark translates Opus to the orchestrator's current Codex model;
lanes inherit that model. Q1 to Q3 reuse their existing worktrees. Workspace
`AGENTS.md` governs output locations: new Cargo output uses stable named
paths under `C:\t\cargo-targets\isometry\`, with a separate lane subdirectory
where concurrent source work requires isolation. The orchestrator grants
one Cargo turn at a time. Temporary scripts stay in the owning worktree and
are removed after use. These current workspace rules replace the older
`target/build` and scratchpad instructions below for this execution.

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

Numbering originally continued at 805; that ruling records the Codex model
translation on resumption. Rulings 806 to 811 settle lineage complexity,
event-handle translation, native deed agreement links, character table
metadata, healing allocation and unknown legacy losses; the next ruling is
812. Form, in the wing
record before the "Two
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

- **2026-10-10, Q3 unchanged seed-0 balance verified:** native Genesis
  `0a55ba9f06247e21f97892468486de8948bedbb2ea313c729809a135cb6b35b5`
  on `ab3b559b` again exhausted the played lineage at tick 115. Founder 17
  received ten meal units, spent twelve upkeep units and provided eight;
  first child 130 received four meal units beside that provision and spent
  twelve upkeep units. The child peaked at nine, below threshold twelve,
  and no born consumer reproduced. Scope blocks and two correctly floored
  shared meals explain sparse intake; no native Refused receipt occurred.
  Conservation and saved replay in both modes passed. Observers were
  removed, exact source hashes restored and the native offline all-target
  compile check passed. Root recomputed the six external evidence hashes;
  findings SHA-256 is
  `d4bde01df88dba6d32bcb958e4669189b7c1cafbc29c28400bcb21bad4f74b23`.
  The findings and raw/session/flow/source files remain under
  `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/`.
  A1 remains 4/20 bare; no rates are promoted. Q3 prepares a declared
  meal-size trial, Q2 prepares native witness compatibility projections,
  and Q8 now owns the serial Cargo slot. Mark's adapter-healing allocation
  fork is pending as ruling 812. Existing trees and targets are reused.
- **2026-10-10, ruling 811:** Mark chose to load legacy losses, heal only
  losses with recorded allocations and retain unknown losses until an
  explicit repair. Unknown cells do not stall known-loss healing elsewhere
  in that body. Q8 carries this into its known/free/unknown representation,
  accounting, save/replay and compatibility gates; ordinary digest/witness
  checks remain intact. Q2 also qualifies the distinction between preserved
  JSON/genesis bytes and changed postcard witness bytes for optional table
  metadata. Neither lane claims archive compatibility from field decoding
  alone. Q3 owns Cargo for the unchanged seed-0 diagnosis; Q2/Q8 prepare
  source. The next ruling is 812.
- **2026-10-10, Q15 refresh verified:** the three query setup paths now use
  pinned Mere's `refresh_queries`; actual motion timestep/gravity and tests
  are unchanged. Tactile passed 4/4, the compiled omitted-refresh control
  failed, exact bytes were restored and its positive passed. Eponym passed
  13/31 (all twelve contact/action checks and one compatibility refusal);
  eighteen known-baseline fixture/admission/archive failures occur before
  changed setup and remain Q5's gates. Mesocosm and Eponym/all-features
  workspace/all-target compile checks passed offline. Raw results/commands
  and hashes are in `Code/testing/isometry/after-pass-2026-10-10/q15-refresh/`;
  `results.json` SHA-256 is
  `727da4f4120c058d81e500098836f2edeb22ed9ab13c8896bd2520a6a0354129`.
  Q15 is done and Cargo is released to Q3's unchanged-seed-0 trace. Q2/Q8
  continue source work in their existing trees; Q8's legacy loss-allocation
  fork is pending as the next ruling, 811. No new isolated resource.
- **2026-10-10, rulings 809 and 810:** Mark chose optional native Character
  table metadata for the authored cell and owner, saved and replicated
  without control or body movement (809), and healing to restore the lost
  cell's recorded previous function or free-pool status (810). Q2 and Q8
  resume source work; their implementation and verification remain open.
  Q3's seed-0 diagnostic is source-ready and parked while Q15 owns Cargo.
  Q2, Q8 and Q15 are the three active lanes; no new worktree or Cargo home.
  The next ruling is 811.
- **2026-10-10, Q3 fresh bare arm rejected:** the unchanged threshold-12,
  one-unit-meal, 80 to 120-tick lifespan candidate ran all twenty bare seeds
  on `ab3b559b`, after prey timing and Q8's native checkpoint integration.
  Only seeds 7, 14, 15 and 19 qualified (4/20). Every seed produced a true
  played-lineage birth and born heirs; all conserved matter each tick and
  replayed in both modes. Sixteen lineages died before tick 180. The bodied
  and fresh original-control arms remain withheld; no rates were promoted.
  The full keyed Genesis/output receipt is
  `Code/testing/isometry/after-pass-2026-10-10/q3-reproduction/q3-reproduction-after-prey-matrix.log`,
  SHA-256 `6b4f2252c4cd438380f7aaad38298345b0fd63d15f91e5c9f4295ca5c9ec4540`.
  Q3 prepares an unchanged-seed-0 founder/firstborn receipt diagnosis while
  Q15 owns the serialized Cargo window for query-refresh verification.
  Q2's metadata and Q8's healing forks remain unanswered; next ruling 809.
- **2026-10-10, Q8 bounded native checkpoint integrated:** `9b0ae46d`
  supplies native hazards, fragment matter/flow conservation, rot and
  explicit crowd scope guards. All 27 code/test/script byte hashes match
  its final tested envelope. Its native all-target check, sixteen harm
  tests, route-cut and founding positives passed; the compiled guard fault
  was caught and restored. Root downstream all-target checks passed offline
  for root/all-features, Mesocosm, Eponym/all-features, wing-integration and
  parry-ground. Exact commands/log hashes are in `q8-root-integration.json`
  under `Code/testing/isometry/receipts/2026-10-10/isocosm/`, SHA-256
  `ceb4640437ed444e40b23f540d03fd62b85e5bfc0214d32c8c1453a9bff4aefa`.
  Ordinary FRAGMENT founding changes genesis digests; ordinary HEAL draws
  are deferred and the opt-in probe marker is dormant. Healing/revival,
  remaining faults and independent crowd certification remain open.
  Q2 and Q8 are parked at their unanswered structural forks; next ruling
  remains 809. Q3 gets the fresh declared reproduction window on this
  integrated source; Q15 gets its bounded source refresh on main. No new
  worktree is needed because Q15's files have no current collision.
- **2026-10-10, Q2 overlay checkpoint integrated:** the complete twelve-path
  vocabulary/helper delta through `702ab6ca` is extracted from Q2's common
  base, preserving its unfinished consumer ancestry. All committed overlay
  bytes match the lane. The root workspace/all-targets offline check passed
  in 3.09 seconds; the lane passed in 4.86 seconds. Its unchanged code/test
  snapshot passed 46/46 earlier, with only README scope amended afterward.
  Native event-key mapping is shared under 807. Product routing, character
  cell/owner retention, V2 and E3 remain open; Q2 is parked at the metadata
  fork with its six-sortie batch prepared. Q15's query-refresh preflight is
  complete and awaits source work after Q8 integration.
- **2026-10-10, Q3 prey-timing checkpoint integrated:** `73c6451f` keeps
  one chosen process per tick while resolving automatic feeding prey once
  at that pass's start. An applicable Thing/Act nudge keeps its named prey;
  actual target/place determine which nudges are answered. All 28 focused
  checks passed (five new controls, six choice, ten feeding and seven
  scheduler); a compiled disabled fix failed the moved-prey control, then
  exact production bytes were restored and the positive control passed.
  The orchestrator's merged native workspace/all-targets check passed
  offline in 22.03 seconds, and weave verified all five merge files. The
  restored and merged `choice.rs` SHA256 both equal
  `bd4759f265eea2f06eb8b8f683edc26dc9302667d8b0fe57226aa61f19b1f16b`.
  Raw control/gate logs are retained with Q3's external receipts. Generator
  defaults and the reproduction harness are unchanged on main; A1 and Q4's
  dependency stay open. Q8 next verifies its bounded crowd guards before
  Q3's declared fresh reproduction matrix.
- **2026-10-10, Q2 native checkpoint integrated:** the twelve native paths
  from common base `c6bc153a` through verified lane checkpoint `9dd71d78`
  are applied independently of the branch's saved consumer WIP. They carry
  ruling 808's typed agreement link, asserted map/storylet/pack records,
  atomic folding, event-key ownership and witness updates. The merged native
  workspace/all-targets check passed offline in 25.39 seconds; the lane's
  focused event compatibility and assertion tests passed 1/1 and 7/7.
  The native lock's SHA256 remains
  `9417e959e1e6b254e4f240ee10e7c7e639b642c71bca3b1459386684aa029321`.
  Q2 remains active for its consumer and overlay gates and the character
  metadata fork. Q8's restored independent harm suite passed 10/10 and both
  planted controls were caught; healing and full C10 remain open. Q3 gets
  the next Cargo turn for its bounded prey-timing tests.
- **2026-10-10, Q3's stale-prey diagnosis:** three short seed-0 probes
  identify automatic prey cached before movement, then refused outside the
  feeding scope. The production recorder was removed byte-for-byte and raw
  receipts retained under `Code/testing/isometry/after-pass-2026-10-10/`.
  Rulings 454 and 683 settle the bounded correction: retain one chosen
  process, draw automatic prey from its feeding pass's start, preserve an
  explicitly named prey and recompute nudge answers for the actual target.
  Q3 implements that correction while Q8 owns a short independent
  hazard/severing/rot check and focused-test batch. Healing allocation and
  character metadata remain pending; no new ruling or rate promotion.
- **2026-10-10, Q3's second candidate rejected:** the declared threshold
  12, one-unit meals and 80 to 120-tick lifespan profile completed 20 bare
  seeds, with only 5 qualifying (1, 3, 14, 15, 17). Conservation and both
  save-replay modes passed all 20. Its bodied and fresh original-control
  arms were correctly withheld after the failed bare gate. Defaults stay
  unchanged, Q3 stays active and Q4 stays gated. The next Cargo turn is a
  narrow native receipt/held diagnosis before another candidate declaration.
- **2026-10-10, Q7 integrated and Q8 launched:** `2c86e5f7` moves Mesocosm
  bodies onto Mere's pinned tenant, with caller-encoder submission, palette
  UVs, configurable ambient light, tracer depth and Netrender layering.
  `LiveBody` remains for real VTT/Eponym consumers and posed queries. Both
  CPU clipping/projection controls passed. The orchestrator rechecked all
  four merged workspaces offline: default Isometer, Mesocosm, root and
  Eponym (the latter two with all features). The ignored Mesocosm lock was
  copied and its SHA256 verified. GPU/headed and performance gates remain
  on A4. Q8 started with read-only harm preflight and takes over the same
  worktree after synchronization; no new worktree or Cargo home is needed.
  Q2's independent short batch passed 1 native event test, 7 assertion
  tests and 46 overlay tests. Q3 receives the next Cargo turn for its
  declared reproduction candidate and unchanged original controls.
- **2026-10-10, Q3 checkpoint integrated:** `e7af61ec` is integrated after
  the merged Isocosm workspace/all-targets offline check, source ceiling,
  staged diff and weave checks passed. Its 42 focused tests certify the
  generated-body accounting and turn-order slice. Q3 remains active for
  the reproduction survival gate; no generator defaults were changed.
  Q2 now holds the serialized Cargo slot for contracts and ruling 808.
- **2026-10-10, ruling 808:** Mark chose a typed optional agreement id on
  native deed events, retaining causal-event validation and the agreement's
  state owner. Q2's Form/Exercise/End tests exposed the former invalid
  `agreement:<id>` cause; its narrow native repair and replay gate follow.
- **2026-10-10, Q3 checkpoint:** `e7af61ec` verifies native anatomical
  account gates, meals and paid births, plus 806's living repertoire order:
  42 focused tests and the Isocosm workspace/all-target check passed. This
  slice is ready for integration; the lane's reproduction survival gate
  remains open and generator defaults are unchanged. Q7 holds the next
  serialized Cargo batch; Q2 and Q3 continue their remaining gates.
- **2026-10-10:** queue written; Q1 to Q3 have work on their branches.
- **2026-10-10, resumed:** remote and local main both verified at `8c8b49bb`;
  all three lane worktrees clean at their WIP tips. Q1 (plans), Q2
  (contracts) and Q3 (reproduction and turn order) relaunched with the same
  briefs, current Codex model (805), serialized offline Cargo, and the
  current workspace output rules. Their WIP remains unmerged pending each
  lane's verification.
- **2026-10-10, first fork round:** Mark settled Q3's metabolic-complexity
  aggregation (806: distinct functions and realized systems across all
  living members) and Q2's event identity (807: shared adapter mapping,
  agreement validated against the native key). Both choices sent to their
  running lanes; implementation and gates remain in progress.
- **2026-10-10, Q1 integrated:** `lane-plans` finished at `901116cc`
  (new finishing commits `3523420b`, `901116cc`). Its archives and folds
  under 793, 800 to 802 and all 19 surviving LIVE plans' current-state
  sections are integrated, with current canonical index rows. The lane
  verified 143 new/index links and 43 full source paths. Merge retains
  rulings 805 to 807 and both sides' dated Progress entries. This is a
  documentation gate; implementation, draw and headed gates stay open.
- **2026-10-10, Q7 launched:** the finished Q1 worktree
  `C:\Users\mark_\Code\worktrees\isometry-plans` is reused at main
  `71233197`, with its branch renamed to `lane-tenant`. Q7 owns it while
  moving Mesocosm to the tenant and checking retirement of `LiveBody`.
  This keeps in-progress renderer changes separate from main integration
  of the concurrent contracts and reproduction lanes. Its Cargo output,
  when granted, belongs at `C:\t\cargo-targets\isometry\tenant`. Q2, Q3
  and Q7 are the three active lanes; no new worktree was created.
