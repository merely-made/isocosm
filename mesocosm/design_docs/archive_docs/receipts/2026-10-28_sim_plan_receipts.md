# Sim receipts

## 2026-11-03 — First ecology and adaptation batch

Base: local main `775277e9e700dd0aaaf53205c499b417da0b303e`, initially clean,
at `/Users/markik/Code/repos/isocosm`. Changes remain local and uncommitted.
The three work lanes owned disjoint source paths in that checkout. The app's
managed-worktree creation rejected the nested repository path, so no new
worktree or branch was created. Existing worktrees were left alone.

Scope and done-conditions are sim S5.1 and S10.1. S10.2 is follow-on work.
The first batch supplies a native ratio query, an explicit adaptive deep-time
entrypoint and the persistence observer. It does not implement protected root
stores, seed banks, metabolic scaling, heritable rate bounds or lifetime
learning, and does not close case 1145 or the five-seed two-year gate.

Evidence root:
`/Users/markik/Code/output/cosmograph-policy/2026-11-03-ecology/`.
The final scope/source manifest and logs are retained there. Compiler
intermediates use the stable `cosmograph-policy` lane; the product check used
the existing `isocosm-runtime-tests` lane. A read-only preflight with a 20 GiB
reservation reported allowed, 327.5 GiB free and an estimated 111.5 GiB cache.

### Commands and results

All Cargo work used the guarded launcher. The shell's Homebrew Cargo 1.99.0
rejected the initial `+1.99.1` argument before compilation. Subsequent calls
ran the launcher through `rustup run 1.99.1`; its compiler reported
`rustc 1.99.1 (58cf8f9c3 2026-10-13)`. That failed invocation remains in
`deep-time-tests.log`.

The initial focused deep-time run passed six tests, including admitted policy
replacement and save/replay through native and physical clocks. Its log is
`deep-time-tests-rustup.log`.

The first broad run passed 319 library tests, with 32 ignored, and five of six
query tests. The residency fixture illegally founded a traveler. It now
starts the journey through native movement. Cross-review also found that a
fixed origin denominator would outlive speciation incorrectly; the query now
supports counting the actor's current lineage. The initial failure is kept
in `cosmograph-validation.log`.

Final broad command:

```sh
rustup run 1.99.1 python3 /Users/markik/Code/tools/build_storage.py build \
  --lane cosmograph-policy \
  --cwd /Users/markik/Code/repos/isocosm/shared/cosmograph -- \
  test --locked --offline --lib --test members_per_head \
  --test world_time_year --no-fail-fast
```

Results in `cosmograph-validation-final.log`: 319 library tests passed, 32
ignored; seven query tests passed; seven observer/terrain tests passed, with
the long two-year trajectory ignored. Total: 333 passed, zero failed, 33
ignored. The six adaptive deep-time cases are included in the library total.

The observer's stationary controls deliberately remove automatic processes.
Their 24 native monthly advances qualify sampling and verdict behavior,
never ecological persistence. Native mortality, migration and speciation
provide loss, return, sharp-decline and extinction controls. The receipt
separates trajectory outcome from `root_store_qualified: false`; its trophic
levels are configured traits, not inferred diet classifications.

The automatic ecology/receipt path was also exercised with seed 9, 30
founders, five lineages, native bodies and the laid 2x2 map for one world
day. `WORLD_TIME_DAYS=1`, `WORLD_TIME_POP=30`, `WORLD_TIME_LINEAGES=5`,
`WORLD_TIME_SEED=9` and an absolute `WORLD_TIME_REPORT` selected this diagnostic:

```sh
rustup run 1.99.1 python3 /Users/markik/Code/tools/build_storage.py build \
  --lane cosmograph-policy \
  --cwd /Users/markik/Code/repos/isocosm/shared/cosmograph -- \
  test --locked --offline --test world_time_year \
  a_bodied_world_time_web_holds_two_years -- --ignored --exact --nocapture
```

Result: one test passed. The receipt has two samples, ending at 1,440
elapsed minute ticks; all five lineages retain six members, with 24 native
bodied members and six unbodied decomposers. The trajectory is `inconclusive`,
the first-year verdict is absent and root-store qualification is false.
`seed9-one-day.json` and `bodied-diagnostic.log` retain the actual parameters,
observations, digests and output. This is a driver/receipt diagnostic, not
long-term persistence evidence.

The product check used:

```sh
rustup run 1.99.1 python3 /Users/markik/Code/tools/build_storage.py build \
  --lane isocosm-runtime-tests \
  --cwd /Users/markik/Code/repos/isocosm/mesocosm -- \
  check --locked --offline -p mesocosm-runtime --lib
```

It stopped at dependency resolution because `mesocosm/Cargo.lock` needs a
refresh under `--locked`; no product source compiled. No manifest, pin or
lockfile changed in this pass. `runtime-check.log` retains the error. A green
standalone Cosmograph run is not product-workspace qualification.

Pinned formatting was applied to new tests/modules. Changed Rust files stay
under 600 lines, local Markdown links resolve, `git diff --check` passes,
and `responsa regen`/`responsa check` keep the generated registry current.

## 2026-11-03 — protected root-store continuation, pre-fix control

The preceding local batch was preserved under
`/Users/markik/Code/output/cosmograph-policy/2026-11-03-protected-roots/`
(`initial-scope.json`, tracked patch and initial untracked sources). HEAD remained
`775277e9e700dd0aaaf53205c499b417da0b303e`; no stage, commit or push.
Read-only build preflight admitted a 20 GiB reserve, reporting 337.7 GiB free
and a 121.3 GiB cache estimate at that observation.

The independent integration control was run before production edits:

```sh
rustup run 1.99.1 python3 /Users/markik/Code/tools/build_storage.py build \
  --lane cosmograph-policy \
  --cwd /Users/markik/Code/repos/isocosm/shared/cosmograph -- \
  test --locked --offline --test world_time_roots -- \
  --exact world_time_producers_start_with_twenty_in_store_and_forty_exposed --nocapture
```

Result: exit 101; one executed test failed, with native `(stored, exposed)`
`(0, 60)` against `(20, 40)`. This is a genuine old-source control, not a
manually weakened post-fix arm. The full log, control source snapshot and
result JSON are retained as `prefix-control.log`, `prefix-control-test.rs`
and `prefix-control-result.json` in that output root.

The full matrix is declared in advance as seeds 5, 9, 11, 17 and 31. The
historical records identify 9 and 31 but do not preserve the full old set;
these results will be a new five-seed measurement, not an exact historical
same-seed comparison.

### First protected-root release validation

The first guarded release pass completed with **405 passed, zero failed,
32 ignored** across thirteen test binaries (including the library). All
fourteen independent protected-root tests passed. This source predates the
scheduler-filing and fully above-ground-frond refinements; it is retained
as an intermediate check, not the final long-run build.

```sh
rustup run 1.99.1 python3 /Users/markik/Code/tools/build_storage.py build \
  --lane cosmograph-policy \
  --cwd /Users/markik/Code/repos/isocosm/shared/cosmograph -- \
  test --release --locked --offline --lib \
  --test world_time_roots --test members_per_head --test anatomy \
  --test bodied --test bodied_grazing --test bodies --test births \
  --test feeding --test ecology_accounts --test ecology_rates_907 \
  --test development --test schedule --no-fail-fast -j 4
```

Full retained log: `release-validation.log` in the protected-roots output
root. These are native behavioral and preservation controls; no full-year
or product-host result is implied.

### Scheduler filing pre-fix control

The exact native test
`controls::healthy_plants_are_not_death_evaluated_and_last_part_loss_files_death`
was added while production remained unchanged, then run with the same guarded
release lane and `--test world_time_roots ... -- --exact --nocapture`. It
failed as intended: ten healthy minutes performed **280 evaluations, all
blocked**, against an expectation of zero. One test executed; exit 101.
The source manifest/patch in `before-filing-fix/`, `prefiled-control.log`
and `prefiled-control-result.json` retain this witness. The fix is a cache
filing predicate with proper native matter invalidation, not skipped ticks.

### Final native build and frozen matrix launch

The final guarded release command adds `--test world_time_year` to the
first release command. It returns exit 0: **416 passed, zero failed,
33 ignored** across 14 binaries, including 15 root controls and 10 fast
observer/admission controls. The full trajectory remains ignored in this
validation pass and is launched explicitly below. The final log is
`release-validation-final.log`.

`final-build/source-manifest.json` captures 878 source/input file hashes
and the full local patch; they were checked unchanged after compilation.
The `world_time_year` executable was APFS-cloned from the guarded release
output into `final-build/world_time_year`, then both files' SHA-256 was
verified as `f97256f411333c1be2e569625a005cd4d3d570c11c59238be424774d3df43c1d`.
The 21,046,112-byte executable, toolchain and test totals are identified by
`final-build/build-receipt.json`.

A direct run of this frozen native test, with `WORLD_TIME_DAYS=1`,
`WORLD_TIME_SEED=9`, `WORLD_TIME_POP=240` and `WORLD_TIME_LINEAGES=10`,
completed 1440 minute ticks, retained structural root qualification and
reported **inconclusive**, as required. Its log and receipt are
`pilot-seed9-one-day.log` and `.json`. This measures the full founding
scale, not the full persistence horizon.

```sh
python3 /Users/markik/Code/output/cosmograph-policy/2026-11-03-protected-roots/run_matrix.py \
  --binary /Users/markik/Code/output/cosmograph-policy/2026-11-03-protected-roots/final-build/world_time_year \
  --source-manifest /Users/markik/Code/output/cosmograph-policy/2026-11-03-protected-roots/final-build/source-manifest.json \
  --evidence-dir /Users/markik/Code/output/cosmograph-policy/2026-11-03-protected-roots/matrix-01 \
  --parallel 5
```

Every seed runs the native test explicitly with `--ignored --exact
--nocapture --test-threads=1`, population 240, ten lineages, four laid sites,
single-member founders, single-clutch sensing/grazing/nohazard bodies,
and 24 world months. The runner strips inherited diagnostic-span overrides.
`matrix-01/matrix.json`, per-seed JSON/logs and `matrix-01-runner.log` retain
progress and the eventual verdict. No native ticks are skipped.

The native `Session::advance` path does not append journal entries; the
reported entry digest can therefore remain SHA-256 of `[]`. Source/genesis/
rules identity, exact advanced ticks and the terminal native state hash
are the useful identity of this experiment. A full two-year replay of a
saved command journal is not claimed by this driver.

### Scheduled compound-meal pre-fix control

The running frozen matrix prompted a native scheduling check. The exact
`scheduled_world_time_multi_eat_matches_the_authored_six_two_four_meal`
test in `--test scheduled_meals` retains the ordinary WorldTime meal,
with only its period shortened for the control. Native direct action
first demonstrates the authored 6 taken / 2 kept / 4 shed. The scheduled advance
actually takes 8 and keeps 4 while shedding 4. One exact test fails, exit 101,
with conservation intact; this is quota misassignment, not matter creation.
The fixture is run through the guarded release lane, both execution modes
being asserted by the test after the repair.

`before-meal-fix/source-manifest.json` and its patch retain the source.
`premeal-control.log` and `premeal-control-result.json` retain the genuine
failure. The five Matrix 01 trajectories continue against their frozen
binary as a pre-correction baseline. Their outcome cannot qualify corrected
scheduled intake. A new frozen build and full five-seed matrix are required
after the transient per-effect quota repair; no biological rate, food port
or digestion policy is changed for this correction.

### Corrected native qualification and second matrix

The native meal-quota correction passes **448 unique tests**, with zero
failures and 33 ignored, across 18 selected suites. The broad release gate
(`meal-fix-validation-final.log`) passed all 17 other targets and eight meal
controls; the new defence fixture first failed admission because of an
incomplete catalogue and an eight-cell part holding eleven cells. Those
fixture defects are retained in the failed logs. Only
`tests/scheduled_meals/defence.rs` changed afterward: it uses the complete
native catalogue and a 27-cell part, asserting its capacity. The final
`--test scheduled_meals --test world_time_year` gate passes all 9 meal
controls and 10 fast trajectory/admission controls; full log
`qualified-focused-validation.log`. Production file hashes are unchanged
between the broad and final focused gates. Counts are deduplicated by
test target in `qualified-build/build-receipt.json`.

Both the original and corrected source also pass guarded release library
checks with `--no-default-features --locked --offline`; the corrected log
is `corrected-featureless-check.log`. These compile the society-free sim
core; they are not a Mesocosm host build. The previous product lockfile
block remains separately recorded, and no Cargo lockfile was changed.

`qualified-build/source-manifest.json` captures 883 unchanged inputs. Its
20,830,432-byte native test executable was APFS-cloned and hash-verified:
`d6ca5a26f0bf2324e18fe6f4ad1de008294f191998c924814cfc95162767ce04`.
The second full matrix uses the same runner command as Matrix 01, replacing
the executable/manifest directory with `qualified-build` and the evidence
directory with `matrix-02`. All five seeds run 24 world months again. Their
founding parameters, initial samples and state/genesis/rules identities
match Matrix 01 exactly; executable/source identities distinguish the
corrected transient quota evaluator.

The original preserved executable is a baseline, not qualification of the
corrected evaluator. Existing histories affected by the former feeding
apportionment can replay differently and hash-refuse. No compatibility
mode or arbitrary archive migration is qualified. The independent
experimental crowd/probe evaluator still has a scalar meal portion; the
new multi-meal qualification is native individual/grouped execution.

### Read-only early-loss diagnosis while the matrices continue

The independent diagnosis lane inspected existing seed-9 receipts and twelve
source files, all hash-matching the corrected frozen manifest. Its retained
note and extracted facts are
`Code/output/cosmograph-policy/2026-11-03-protected-roots/seed9-lineage3-readonly-diagnosis.md`
and the sibling `seed9-lineage3-readonly-diagnosis-facts.json`.
At capture the corrected observer first lacks lineage 3 at month three
(131,490 ticks) and lineage 8 at month eight (350,640 ticks); both are first
absent at month two in the baseline. No new world was launched for this
diagnosis, and it is not the final matrix result.

The note establishes the mismatch between founded producer-only intake and
the role-derived breeding numerator. It also derives the nominal exposed
part allocations from the recorded recipes: producer 0 has a largest share
of four, while producer 5 has two shares of six. The predator-role meal's
second `Eat` and following transfer request six. Each effect reselects from
current staged holdings, so the source gives availability bounds rather
than an observed success frequency. Worldwide event counters do not
attribute an individual lineage's deaths. The proposed three-month seed-9
observer experiment is follow-up work, not executed evidence or a rate
change in either frozen matrix.

### Corrected twelve-month checkpoint

All five corrected worlds reached 525,960 native minute-clock ticks and
retained their exact monthly samples. Seeds 5, 11, 17 and 31 pass every
original first-year predicate; seed 9 fails after the recorded lineage-3
and lineage-8 losses. The baseline has the same first-year pass/fail split.
Both matrices remain in progress for the full 1,051,920-tick horizon, so
these are first-year observations rather than completed two-year verdicts.
The corrected checkout still matches all 883 captured source input hashes;
all 48 changed Rust files remain at or below 600 lines, and
`git diff --check` passes.

The independent integer recomputation agrees with every native first-year
verdict. `corrected-firstyear-checkpoint.json` retains months 0 through 12,
failed transitions and capture/source identities; SHA-256
`f909eb7cfc26fb60d3c63fe30e5f87bffa8df6affc7f24fa39dd5357fce01c43`.
Both lost seed-9 lineages fail alive-at-month-12, the fourfold window and
the no-third-loss condition. This checkpoint is retained under the same
protected-roots evidence directory as the matrices.

### Baseline second-year losses before completion

At month 22 (964,260 elapsed ticks), Matrix 01 seed 11 first lacks founding
lineages 3 and 8. Their preceding month-21 counts were six and four. That
seed passed every original first-year predicate, so these are newly
observed second-year losses. They do not create a configured regional-level
loss. The corrected seed-11 run has not yet reached month 22 at this
observation; the completed comparison remains pending.

The same native samples expose a compartment change. At month 18, producer
0/5 root tissue is 886/1,573 and exposed tissue is 4,257/5,739 across
116/154 living members. At month 22, root tissue is exactly zero in both,
while exposed tissue is 916/2,001 across 87/123 living members. These are
registered tissue readings, not a reconstruction from whole-body biomass.
Global site soil rises from 32,038 to 44,434, so a blanket global-soil
depletion explanation is unsupported. Local access, production constraints,
meal refusal frequencies and individual death causes remain unmeasured.

Matrix 01 seed 31 first lacks the same two lineages at month 23
(1,008,090 elapsed ticks), after passing every first-year predicate. Both
were present at month 22. No configured regional level is lost. Its
corrected counterpart is still at month 16 at this observation, so an
equal-time correction comparison remains pending.

Matrix 01 seed 17 first lacks lineages 3 and 8 at month 21 (920,430
elapsed ticks); their preceding month-20 counts were eleven and five.
That seed also passed the first-year predicates and retains its configured
regional levels. At this observation four baseline seeds have recorded
lineage failures, with seed 5 and the corrected full matrix still running.

Matrix 01 seed 5 first lacks lineages 3 and 8 at month 23 (1,008,090
elapsed ticks), after counts one and three at month 22. Its first-year
predicate had passed, and its configured regional levels remain present.
All five baseline worlds have now recorded a bar failure. At this
observation seeds 11/31 have completed all 25 samples and the full horizon,
with terminal native state hashes and no advancement failure; the other
three baseline worlds and all corrected worlds continue.

### Equal-time late prefix under the correction

Seed 31 at month 19 (832,770 elapsed ticks) has corrected producer-0/5
registered root totals 1/0 across 82/118 living members; baseline values
are 693/279 across 152/138. Corrected exposed totals are 1,488/2,218 versus
baseline 5,389/4,792. The one remaining root unit must not be described as
an exact zero after rounding population means. Nominal predator-3/8 counts
are 7/6 corrected versus 11/19 baseline, and global site soil is
44,429 versus 32,249. Thus the correctness repair is not evidence of a
general ecological improvement. These equal-time samples still do not
measure individual death causes or feeding-refusal frequencies.

At corrected month 21 (920,430 elapsed ticks), seed 31 first lacks lineages
3 and 8, after counts five/five at month 20. Its first-year bar passed;
its second-year bar now fails, two observed months before the baseline's
first absence. Configured regional levels are still present. This is the
first newly observed second-year loss under the correction, in addition
to seed 9's early losses. It establishes that the quota repair has not
resolved the measured persistence problem; the complete horizon continues.

Corrected seed 31 additionally first lacks lineage 9 at month 22 (964,260
elapsed ticks), after seven members at month 21. Seven founding lineages
remain at that sample, and the configured regional levels remain present.
The observer and completed export check all ten founding lineages; their
loss list is not limited to the original nominal predator pair.

### Completed baseline matrix

All five Matrix 01 worlds complete exactly 1,051,920 native minute-clock
ticks with 25 observations, terminal state hashes and no advancement
failure. All five fail the asserted two-year criterion; the runner exits
1, and its terminal `matrix.json` SHA-256 is
`9363d5c51db7ed611618c3b1d9039046d4690ed735d973460216732f40d56f59`.
Every seed has eight founding lineages still living at month 24, with
lineages 3 and 8 absent. No configured regional-level loss is recorded.

| Seed | First-year bar | First observed absence of 3/8 | Living members at month 24 | Two-year outcome |
| --- | --- | --- | ---: | --- |
| 5 | Pass | Month 23 | 273 | Fail |
| 9 | Fail | Month 2 | 796 | Fail |
| 11 | Pass | Month 22 | 586 | Fail |
| 17 | Pass | Month 21 | 831 | Fail |
| 31 | Pass | Month 23 | 428 | Fail |

The corrected matrix remains in progress; this completed baseline is not
qualification of the corrected evaluator or a general claim about the
effect of root stores. Both matrices already include the protected
root-store implementation and differ in native meal-quota execution.

### First corrected terminal results

Corrected seed 11 completes the exact 1,051,920-tick horizon with 25
observations, all ten founding lineages present and a passing two-year
verdict. Its baseline failed after first losing 3/8 at month 22. Corrected
seed 31 also completes, failing with seven founders still living after
3/8 were first absent at month 21 and 9 at month 22. These opposite
directions make the ecological comparison mixed. The remaining corrected
worlds still complete their full horizons before the final export.

Corrected seed 5 first lacks lineages 3 and 8 at month 20 (876,600 elapsed
ticks), after counts ten/eleven at month 19. Its baseline first lacked
them at month 23. The corrected first-year predicate had passed; its
second-year predicate now fails, with configured regional levels still
present. Seeds 5/9/17 continue to the full horizon, and seed 17 retains
all ten founders at this observation.

Corrected seed 17 first lacks lineages 3 and 8 at month 21 (920,430
elapsed ticks), after counts three/one at month 20. This matches the
baseline's first observed loss month. With seed 11's completed pass, all
five criterion outcomes are now determined: one pass and four observed
failures, while three corrected worlds still finish their exact horizons.
No configured regional-level loss is recorded; the five-seed prerequisite
remains uncleared.

Corrected seed 9 additionally first lacks lineage 9 at month 21 (920,430
elapsed ticks), alongside its earlier first absences of 3/8 at months
three/eight. Seven founders remain at this sample; configured regional
levels remain present. This additional loss is retained independently of
the seed's already failed first-year predicate.

Corrected seed 9 then first lacks lineage 4 at month 22 (964,260 elapsed
ticks), after seven members at month 21. Six founders remain: 0/1/2/5/6/7.
Its configured regional levels remain present. Thus the complete loss
record includes four founders even though its first-year bar was already
failed by the original pair.

## 2026-11-03 — completed paired two-year result

All ten native worlds complete their requested 1,051,920 minute-clock
ticks with 25 exact observations apiece, including founding: **10,519,200
ticks and 250 samples**. Structural founder admission passes every run.
There is no advancement abort. Both matrix runners exit 1 because their
native acceptance assertions record ecological failures. Matrix 01 passes
zero of five; the corrected Matrix 02 passes **one of five, seed 11**.
Both matrices pass the unchanged first-year predicate on 5/11/17/31 and
fail it on 9. The five-seed two-year prerequisite remains unmet.

| Seed | Corrected first observed global absences | Founders living at month 24 | Living members at month 24 | Corrected two-year bar |
| --- | --- | ---: | ---: | --- |
| 5 | 3/8 at month 20 | 8/10 | 96 | Fail |
| 9 | 3 at month 3; 8 at month 8; 9 at month 21; 4 at month 22 | 6/10 | 244 | Fail |
| 11 | None | 10/10 | 420 | Pass |
| 17 | 3/8 at month 21 | 8/10 | 442 | Fail |
| 31 | 3/8 at month 21; 9 at month 22 | 7/10 | 273 | Fail |

First absence is a monthly observation bound, not an exact death tick.
Year-two absence includes founders already lost in year one. No configured
regional-level loss is observed in any world; this refers to the fixed
connected footprint's producer/consumer/decomposer traits. Baseline local
losses/recolonizations total 55/0; corrected totals are 101/10. Monthly
sampling can miss intervening losses and returns.

All five pairs match their initial samples, founding parameters,
genesis/rules digests, initial native hashes, feeding definitions and root
admission. The preserved baseline already contains protected root stores.
The corrected executable changes native meal-quota execution, not the
authored rates, ports or initial world. The ecological effect is mixed:
seed 11 retains its founders under correction, while 5/31 lose their
original pair earlier and 9/31 lose additional founders. This is no general
ecological-improvement claim and no no-root versus root causal comparison.

The final retained export directory is
`Code/output/cosmograph-policy/2026-11-03-protected-roots/comparison-final/`:

| Artifact | Contents | SHA-256 |
| --- | --- | --- |
| `comparison.json` | Per-seed predicates, first losses, final populations, regional/site turnover, monthly tissue/resources, matched initials and source identities | `af176a60aa8c2cc17f326c7ef391edf78244bf10716a4bdac649dba058eb797d` |
| `trajectories.csv` | 2,500 exact integer population/tissue rows; absent populations are zero, unreported tissue is blank | `a0c34d7759ffc3ac337afa8b5aa9881286b62f481922045789915f3e303bd25f` |
| `resources.csv` | 1,000 monthly site soil/detritus rows | `e5c0a320bb8225b76d5087b5574aa4612885b5e542592b27a8decf02a9df4cb0` |

The corrected terminal `matrix-02/matrix.json` SHA-256 is
`2c3a9dedeee716a425ad674b10c692c919619f525d0a03806e29f15c8f2ce59b`.
The final helper verifies terminal receipt/log hashes and both frozen
executables, source manifests and captured patches. Its literal integer
recomputation agrees with every native first-year and two-year verdict.
`independent-final-comparison-audit-v2.json` separately recomputes all
250 samples and checks every CSV field, row uniqueness and identity. The
retained v1 audit records a corrected checker-schema assumption, not an
export discrepancy. `independent-qualified-build-audit.json` independently
confirms the 448/0/33 deduplicated native qualification and all 883 current
source inputs.

The source-derived food-contract diagnosis remains open. Every nominal
animal has a producer-only intake, while role-based breeding counts may
name different prey. Whole-body mass can pass a target gate whose selected
part cannot fund a complete compound meal. Native tissue measurements show
both depletion and renewal: corrected seed 31 has 1/0 registered root units
at month 19 and 763/856 at month 23, while its lost founders remain absent.
These are living-population readings including recruits, not tracked
adult-store trajectories or per-lineage death-cause measurements. The
proposed unchanged-rule three-month seed-9 observer experiment is recorded
in the owning plan and has not been executed.

Qualification remains native compartment ecology over saved anatomy.
Articulated terrain placement, arbitrary old-save migration, experimental
crowd/probe multi-meal parity and the Mesocosm host build are not qualified.
The existing host lockfile block is unchanged. Full two-year save/replay
was not run: regression replay controls pass, and the long receipts retain
genesis, exact native ticks, final state hashes and evaluator identities;
their pure-time command histories can be empty. CSV living biomass and
site resources do not constitute a global mass audit including dead bodies.
No seed bank, metabolic-scaling, lifetime-learning or people phase is
started by these failed persistence results.

## 2026-11-04 — unchanged-rule food diagnostic begins

The authorized S10.3 continuation preserves the local patch at
`Code/output/cosmograph-policy/2026-11-04-food-contract/initial/` before
instrumentation. All 883 source inputs match the preceding qualified build,
at unchanged base commit `775277e9e700dd0aaaf53205c499b417da0b303e`.
The prior frozen qualified binary supplies the independent exact-three-month
oracle; no new rules, rates, feeding ports or random draws are introduced.

The oracle runs the existing `a_bodied_world_time_web_holds_two_years`
ignored test with `WORLD_TIME_SEED=9`, `WORLD_TIME_MONTHS=3`, and report path
`unchanged-oracle-seed9.json` under the new retained evidence directory.
It completes 131,490 ticks, exits zero and reports `inconclusive` for the
two-year criterion. Its four native samples exactly match the preceding
Matrix 02 seed-9 prefix. Terminal state hash: `16509682181590951978`.
Receipt SHA-256:
`1e0f6a20f605f4f24d5cbf16080efd8e46c2d64b1dd42768f05ac8355dd98853`.
`oracle-identity.json` binds the frozen binary/source and full reference.

The instrumentation and harness are separate file-owned lanes. The former
observes attempts/refusals and birth opportunities before filters; the
latter must reproduce the oracle's rules, genesis, samples, final state and
history identities. Independent food-contract analysis and regression
controls form the third lane. Compilation, neutrality checks and the
observed run are still pending at this entry. A native connected-versus-
disconnected recovery control is also being prepared, explicitly qualifying
movement and observation rather than long-term ecology.

### Unchanged observer completed and reconciled

The observation-only frozen executable is
`diagnostic-build/world_time_food_diagnostic`, SHA-256
`080692645ccf1361535032495fb16754fef5d4a2eb42be37188df54291313d1d`.
Its final focused gate passes nine diagnostic tests (one full run ignored)
and the connected/disconnected recovery test. The independent prospective
alignment test fails as intended: freshly generated predator 3 does not
admit living animal matter. Full log:
`diagnostic-final-controls-and-prefail.log`. The source snapshot verifies
892 inputs and captures the intentionally pre-fix control.

The ignored observation runs all 131,490 native ticks and exits zero. All
four complete samples, initial/header fields, terminal state hash
`16509682181590951978` and history digest exactly equal the independent
frozen oracle. `unchanged-observer-comparison.json` independently verifies
the native identities and each lineage's birth/death balance in every
monthly interval. Observed receipt SHA-256:
`a641945df146eff147dbdc1027f50c8690e9d9c0572cf56d3a7ea58fced35390`.
Comparison SHA-256:
`0953ea7a2917299dba21329c470c8ab125a4dbf47c29c79d066689300bbc2c7b`.
`unchanged-food-summary.json` derives frequencies from complete native
aggregates; detailed bounded examples are not frequency estimates.

| Observed quantity, all three months | L3 | L8 |
| --- | ---: | ---: |
| Committed feeds, all on producer 5 | 36 | 73 |
| Planning refusals at fixed waste transfer | 322 | 297 |
| Committed starvation deaths | 9 | 3 |
| Committed hazard deaths | 0 | 0 |
| Native births | 0 | 1 |
| Due birth opportunities while alive | 0 | 3 |
| Due opportunities with sufficient tissue | 0 | 1 |

L8's three opportunities pass age, role-count and whole-body port-count
gates; only one passes tissue and commits a birth. L3's nine deaths therefore
have native starvation-process evidence, while its role-derived breeding
numerator is not shown to cause the early loss. No successful predator feed
uses producer 0. A bounded example stages retained3/lost5 and then refuses
Transfer6 for insufficient detritus, committing none of that staged work.
Attempt frequencies begin after ordinary scheduler filters. Whole-body
port-member counts include protected tissue and are not an available-part
meal count; speculative amounts are not actual food consumption.

The completion releases the alignment phase. The selected design retains
the existing one-third extraction through conservative actual ingesta,
one admitted part take and a bounded remainder, shared with target and food
reads. Roles seed ports, and port/provenance permissions govern thereafter.
The life-history table, landscape and seed are not tuned. The post-change
instrument has its own explicit identity and the same three-month horizon;
the frozen old observer and all earlier persistence outcomes are retained.

An independent audit reconciles all 30 lineage/month life equations and
all three global balances. The six live-feeding animal lines take exactly
6,489 units of `matter:0-0` and 10,443 of `matter:5-0`. The older summary's
`prey_lineage` graph records selected hosts, not source-account provenance:
decomposer meals targeting dead L3/L8 take 27/9 loose `world:soil` units,
with no fauna-origin matter from either host class. This qualifies the
earlier host-graph interpretation without changing immutable exports.
The aligned analyzer classifies every committed raw take through its saved
account's `Matter.lineage`, kingdom and traits; host identity and liveness
remain separate fields. Retained/waste outputs remain actor/process totals,
not current-source edge allocations, because previous gut residue may mix
origins. The normalized `world:ingesta` account has foreign world-ground
provenance; a corpse reading cannot reconstruct its original food class.
After live predation is admitted, `ecology:death` identifies native
own-tissue depletion, not a uniquely metabolic cause.

### Native alignment and final focused gate

Production alignment uses optional `PortFeeding.meal` metadata, one actual
admitted part take, conservative digestion and `FoodPerHead` over the same
eligible exposed holders. The ordinary new WorldTime Diets ports seed
herbivory, animal feeding (including other predators) and mixed feeding.
Actual port declarations and source provenance own permission thereafter.
No life-history rate, founding, landscape or random seed is tuned.

The first broad aligned gate, `alignment-release-validation.log`, passes
all 14 independent food-contract controls, all 319 library tests and all
15 root-store controls. Its four failures are retained: three old scheduled
fixtures still expected the replaced compound-meal policy, and a diagnostic
fixture looked for foreign ingesta with the owned-tissue physiology reader.
The diagnostic correction reads native part ledgers directly. The scheduled
fixtures now explicitly author their original producer-only, two-Eat,
fixed-transfer meals with omitted partition metadata, retaining their
original scarcity/refusal and quota assertions as compatibility controls.
New default meals are covered independently, rather than weakening those
legacy quota checks.

The final focused gate also includes participant and native in-transit prey
exclusion, plus refusal to designate current or ancestral owned tissue as
foreign ingesta beside an admitted non-world foreign-matter control.
`aligned-qualified-build/source-manifest.json` and `source.patch` capture
the final compiled inputs before that gate; the first aligned attempt's
snapshot remains separately preserved in `aligned-build/`. The final gate
and the post-alignment three-month run are pending at this entry.

The regional recovery control uses native `Kill` and `Move` over a finite
surviving source population, with connected and disconnected route arms in
both native execution modes. It checks actual migrant IDs, conserved
matter, unchanged global living count and save/replay. It demonstrates a
controlled recovery mechanism, not autonomous migration policy or long-term
ecological recovery. Undigested ingesta on dead parts is measured separately and can enter the
legacy decomposer corpse meal. An authored native Rot preserves its account
when releasing it to a site, but this generated world defines no Rot. Its
site detritus process does not consume site-dropped ingesta. Protected roots
also remain excluded from corpse bites. Automatic decay and that recycling
policy remain open; conservation alone does not demonstrate mineralization.

The first focused gate passes all 17 then-current food-contract controls,
all 11 aligned diagnostic controls and regional recovery. Three authored
legacy fixtures still fail because their new helper incorrectly assumes
the body has only one port; actual bodies can declare two. The helper now
addresses the same supported function and lowest living part as native
feeding. The next gate, `alignment-query-prefail.log`, passes all nine
legacy meal tests and the existing 17 contract controls.

Its sole failure is an additional genuine admission control: a selectorless
manual partitioned meal is usable and conserved, but a `FoodPerHead`
reference to that unreadable feed is wrongly admitted. The snapshot is
`before-query-selector-guard/`. A minimal validator check now refuses that
reference while preserving the manual meal. The final contract/diagnostic
rerun is `alignment-final-validation.log`; `aligned-final-build/` is the
new source snapshot for its eventual frozen executable. Earlier snapshots
are preserved, and the final capture declares only the seed-9 protocol.

### Final frozen alignment and comparison launch

The final contract/diagnostic release gate exits zero: 18 independent food
controls and 11 fast diagnostic/admission controls pass; two diagnostic
entry points remain ignored until invoked explicitly. The selected broad,
focused and pre-fix gates deduplicate to **438 passed, zero failed, 35
ignored across 14 targets**. `aligned-final-build/build-receipt.json`
selects the latest result for each target and lists all nine compiled
inputs changed after the broad gate. The final contract and observer are
rebuilt/rerun after the validator repair; the other selected regression
results retain their actual earlier gate identities.

All 904 final compiled input hashes match the captured source. The frozen
16,606,624-byte executable is APFS-cloned and hash-verified:
`bc3a08457b3733b4e97378fb0d7d8b60ff760fe8e377691ea4e1e311fc5aec22`.
Final manifest SHA-256:
`8d546dbe3ce71ef71486a7de4e10caa3a41d9e003fddb8e04ce251cbecc5847c`.
Final captured patch SHA-256:
`c05eeef726e80623b4cc329afc81bdb53d70b90ef84c40f78a8e0a66352469e2`.

`capture_aligned_seed9_initial_reference` runs explicitly with the final
manifest/patch paths and `FOOD_DIAGNOSTIC_REFERENCE` pointing to
`aligned-initial-seed9.json`. The zero-tick capture exits zero, and the
output-only `analyze_alignment.py --preflight-reference` also exits zero.
It verifies identical initial ecological samples and producer structural
admission against the unchanged receipt, inherited numeric native clocks,
and the new frozen identities. New genesis digest:
`2879f4011a77cd58ab3508f8b94a3349641cc0b27d44b25dbd06899a79721bb1`;
rules digest:
`84f6ada57bdaa7dcb9303614e8687f82865edd40717822fc2803dde25a1671b5`;
initial native state hash: `4192256976755276955`.

The same frozen executable then runs
`aligned_seed9_food_contract_for_three_months --ignored --exact --nocapture
--test-threads=1`, with `FOOD_DIAGNOSTIC_REPORT` pointing to
`aligned-observed-seed9.json` and the same three identity environment
paths. The run advances all 131,490 native ticks, publishing atomic progress
and exact monthly boundaries; it is still running at this entry. The
featureless release library check is separately underway. No duplicate
world, new seed set, life-history tuning or two-year rerun is launched.

### Completed aligned three-month measurement

The frozen aligned run exits zero after exactly 131,490 native ticks and
423,309 ms. Final native state hash: `5599691416460810812`. The history
digest remains SHA-256 of `[]`; pure time advances do not create a command
journal. The receipt and frozen identities qualify the native trajectory;
a full three-month saved-journal replay is not claimed.

`analyze_alignment.py` completes exit zero, verifying the exact span,
explicit aligned phase, all frozen input/reference identities, equal
initial ecological sample, inherited numeric clocks, bounded partition
examples and every one of the 30 monthly lineage life balances. The
complete endpoint comparison is:

| Lineage | Founded role | Unchanged members, months 0/1/2/3 | Aligned members, months 0/1/2/3 |
| --- | --- | --- | --- |
| 0 | Producer | 64 → 66 → 71 → 80 | 64 → 71 → 74 → 77 |
| 1 | Grazer | 18 → 26 → 29 → 34 | 18 → 23 → 23 → 24 |
| 2 | Decomposer | 19 → 23 → 29 → 39 | 19 → 25 → 33 → 40 |
| 3 | Predator | 9 → 6 → 1 → 0 | 9 → 9 → 9 → 9 |
| 4 | Omnivore | 10 → 11 → 13 → 14 | 10 → 11 → 12 → 12 |
| 5 | Producer | 67 → 69 → 75 → 82 | 67 → 70 → 77 → 83 |
| 6 | Grazer | 10 → 12 → 16 → 17 | 10 → 10 → 14 → 14 |
| 7 | Decomposer | 27 → 34 → 46 → 57 | 27 → 38 → 50 → 63 |
| 8 | Predator | 6 → 3 → 4 → 4 | 6 → 6 → 6 → 6 |
| 9 | Omnivore | 10 → 9 → 13 → 13 | 10 → 10 → 13 → 13 |

Final global living count is 341, beside 340 unchanged. All ten lineages
remain present in the aligned run; L3/L8 have no native birth or death.
Neither run loses a configured regional level in these three months. The
aligned monthly observer records no local losses or recolonizations, so
it supplies no spontaneous recovery witness. The separate finite-source
connected/disconnected native fixture supplies the controlled mechanism.
Monthly endpoints can miss intervening local turnover.

Actual living-target intake through the partitioned fauna feeds is 23,476
producer-origin and 3,902 fauna-origin matter units. Predators L3/L8 take
1,837/1,239 units, all fauna-origin. That includes 34 units from L8 tissue
taken by L3 and 41 from L3 tissue taken by L8, plus same-lineage and other
consumer tissue. These are actual source accounts, not inferred host
roles. Omnivores consume both classes; grazers consume producer-origin
matter. The aligned predators commit 821/544 recorded scheduled meals
with no refused recorded plans or scheduled meals, beside the unchanged
36/73 commits and 322/297 fixed-transfer refusals. Attempts begin after
ordinary scheduler filters; the figures do not count hypothetical meals.

The six partitioned fauna feeds take 27,378 total units, producing 9,098
retained tissue and 18,196 site waste units, with 84 units left in physical
intake parts. Initial ingesta is zero; no animal dies, and all loose entity
balances, dead-part balances and unassigned ingesta pools are zero. Thus
this measured feeding-throughput balance is exact:
`27,378 = 9,098 + 18,196 + 84`. This is not a global whole-world mass audit
or an assertion that retained throughput remains in living tissue. L3/L8
final carries are 13/9 units across 9/6 animals. Outputs remain actor/process
totals because carry can combine food origins.

L3 has no due birth opportunity during either run. Aligned L8 has four,
all age-ready and actual-food-ready but none tissue-ready (observed tissue
14/9/9/8 against the unchanged threshold 23). Unchanged L8 had three, all
passing its food proxies, one tissue-ready and producing one child. Better
short-span survival therefore does not establish predator reproductive
replacement or long-term persistence. The change bundle aligns food
contracts; it does not isolate the ecological effect of ports, counting
and meal carry as separate interventions.

| Final retained artifact | SHA-256 |
| --- | --- |
| `aligned-observed-seed9.json` | `da65903e6202e0cb38795ff529e4930a1957f3588d221bb7210e4a5e680488cf` |
| `aligned-food-summary.json` | `bda4cd8a949470f13612d5363faf6bb9e5dc3fe243dc36016f70e5a26d3c22b1` |
| `aligned-food-comparison.json` | `4ae130a370ded9dd87b4eb1874d8fd81e3774d8e458da8d0dbfb708496ebe428` |

The guarded final release `check --lib --no-default-features --locked
--offline` also exits zero (`alignment-featureless-check.log`). The
independent build audit verifies all 904 inputs, the frozen and cached
executable, each retained gate hash, all 438 distinct passing test names,
35 ignored checks and zero final selected failures; artifact:
`independent-aligned-build-audit.json`.

The original five-seed two-year result remains one pass in five, with only
seed 11 passing. This new three-month run reports `inconclusive` for that
criterion. Under the November 4 steering, further ecology implementation
can address functional variety, conserved recruitment/dormancy and
recycling rather than waiting for every founding lineage to persist. A
longer, diverse-world regional-function criterion remains to be stated and
measured before ecological readiness or people are claimed. Source stays
local and uncommitted; no Mesocosm host, articulated terrain, unsupported
crowd/probe runner or arbitrary save migration is qualified by these runs.


The final documentation/source review passes `responsa regen` and `check`,
all 734 local Markdown file links, `git diff --check`, and the 600-line
limit for all 77 changed Rust files (largest 583). All 904 frozen compiled
inputs still match. The retained report is
`final-source-and-document-review.json`; documentation logs are
`documentation-final-regen.log` and `documentation-final-check.log`.


Independent final audits reconstruct the summary tables directly from raw
native observations without importing the exporter. All 44 source-account
intake edges, three monthly and complete-span tables, actual outputs,
breeding reads/examples, 18 animal-lineage/month bolus balances, 30 life
equations, paired samples/resources, terminal identities and 42 inherited
clock/aging records agree. No raw or export discrepancy is found. Retained
index: `independent-aligned-audit-index.json`, SHA-256
`369ec220dc596871d6175dc625cc4c277154a91b52cdf1bda6c7e13869037008`.
It binds `independent-aligned-build-audit.json`,
`independent-aligned-raw-audit.json` and
`independent-aligned-export-audit.json`. Earlier checker drafts are retained
and their wire-schema normalizations are explained in that index.

## The bar rerun after E7b, 2026-11-08 (cases 1173, 1181)

Branch `lane/bar-rerun`, source `7f87d95e`, release
`world_time_year-2cae1ffaeb67b593.exe`, `WORLD_TIME_MONTHS=24`, one seed at
a time, grouped; receipts and logs `Code/testing/isocosm/bar/2026-11-08/seed-<n>.{json,log}`.
Every run completed 1,051,920 ticks and 25 samples, wrote both verdicts and
then failed its assertion on the bar (exit 101). History digest
`4f53cda18c2baa0c...` (no entries) for all.

| Seed | Genesis digest | Rules digest | Final state hash | Peak living (month) | Wall |
| --- | --- | --- | --- | --- | ---: |
| 5 | `8638cef5271d893e...` | `3b14965d45aa1da6...` | 7186549771894933760 | 935 (19) | 1,341 s |
| 9 | `1bf002d69da8fdd0...` | `84f6ada57bdaa7dc...` | 10045388551275423687 | 888 (19) | 1,123 s |
| 11 | `8a5f7797e30dd430...` | `efe5fce447e5ff83...` | 8081449221894456226 | 583 (11) | 945 s |
| 17 | `5cd0da282113d5ef...` | `a110dca4fa251bc5...` | 7758574018933200066 | 869 (16) | 1,138 s |
| 31 | `8d0e7da30625829b...` | `ea13e2d7acf2e8d6...` | 7726174206338370392 | 542 (10) | 776 s |

| Seed | Comparator (first year, year two) | Bar (1173) | Diets lost and returned | Lineage losses | Living at month 24 | Wall |
| --- | --- | --- | --- | --- | ---: | ---: |
| 5 | Fail: first year passed, all ten lineages gone in year two | Fail: every diet gone by month 21 | the whole web starved between months 19 and 21 (935 living at month 19, 4 at month 20); the predators at site 3 were lost at month 5 and held elsewhere until the collapse | all ten, months 20 to 21 | 0 | 1,341 s |
| 9 | Fail: first year failed (lineage 7), all ten gone in year two | Fail: every diet gone at month 21 | the same collapse: 888 living at month 19, 688 at month 20, none at month 21 | all ten, month 21 | 0 | 1,123 s |
| 11 | Fail: first year failed (lineages 3 and 8) | Fail: predators gone everywhere from month 4 | predators lost at sites 0 and 2 at month 2 and at sites 1 and 3 at month 4; grazers at site 1 lost at month 20, back at month 22 by walking in from sites 0 and 3; producers at site 1 lost at month 12, held elsewhere; grazers at site 1 lost again at month 23, still open | 3 and 8, month 4 | 536 | 945 s |
| 17 | Fail: first year failed (lineages 3, 6 and 8) | Fail: predators gone everywhere at months 23 and 24 | predators lost at site 0 at month 3, site 2 at month 4, site 1 at month 19, site 3 at month 23; seven returns within one to three months: grazers walking in from sites 0 and 3, omnivores from site 3, decomposers and producers by scattered birth; four patches still open at month 24 | 8 at month 19, 3 at month 23 | 262 | 1,138 s |
| 31 | Fail: first year failed (lineages 3 and 8) | Fail: predators gone everywhere from month 2 | predators lost at every site at month 2; grazers at site 1 back in a month (walking in from site 3), decomposers by scattered birth, omnivores walking in from sites 0 and 3; producers at site 1 lost at month 11, held elsewhere | 3 and 8, month 2 | 500 | 776 s |

Seed 9's trajectory beside the per-tick draw at `286a23ec` (same food code,
the EX4 timing run), living / cumulative hazard deaths / cumulative
starvation deaths: month 12, 612/99/497 then and 615/102/502 now; month 18,
626/190/1132 and 874/200/948; month 20, 534/214/1347 and 688/226/1359;
month 21, 504/224/1429 and 0/231/2056.
