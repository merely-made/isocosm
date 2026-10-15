# The stock/title plan: native tests for the society layer

**Date:** 2026-10-14

**Status, 2026-10-14:** in progress. The `society` feature gate (893, 894) and
T1 to T6's placement are landed on `owned-stock-tests`, local and
unpushed; T3's statements, T6's strain and produced references, and T7 are open.

Ruling 882 put titles outside C11: titled transit and owned-stock custody
are host and player commands, native-tested as a separate boundary owned by
this plan. Ruling 886 parks the society layer (projects, institutions,
custody, party, goals, polities) behind a feature, 893 gates its operations
and keeps its state, and 894 gives this plan the layer's native tests and the
gate. C11's crowd keeps refusing owned stock, custody and titled transit;
nothing here reopens C11. Tuning is open (888) and the test deferral of 826
is lifted for this surface.

The surface is the post-826 source in `shared/isocosm`: `src/projects/**`,
`src/spread/owned.rs`, `src/custody/**`, `src/party/**`, `src/institutions/**`,
`src/goals/**`, `src/polities.rs`, and the consumers of a
`projects::DebitSelection`. At the 826 base `430d2e49` it had one test,
`tests/checkpoint11_spread.rs::normative_direction_requires_exact_owned_quantities_and_retains_claims_through_delivery`.

## Method

Each phase adds native tests under `shared/isocosm/tests/`, one file per
subject module (600-line ceiling), against explicit test-world inputs. Every
test checks, where it applies:

- **Conservation:** `Simulation::matter` is unchanged across reserve,
  debit, disposition and carry, counting project reserve and construction
  once.
- **Refusal:** a refused command leaves the state hash and the project
  records byte-identical.
- **Replay:** the save, replayed from genesis in both execution modes,
  reaches the identical state hash and project records.

Each phase also adds compiled source faults to
`tests/owned_stock/controls.py`, in the C10/C11 style: apply one declared
byte span under an exact SHA, show the named test fails, restore the exact
bytes. A phase is done only when every fault it adds is caught and restored.

## Phases

## Phases

### T1. Owned stock and projects (837 to 840, 843, 845, 846, 849, 850) — landed

Tests in `tests/owned_stock.rs` and `tests/owned_stock/`:
`ownership.rs`, `work.rs`, `consent.rs`, `placement.rs`, `debit.rs`, with the
bench world in `fixture.rs`.

Done when:

- assertions name exact lots that never exceed actual matter, and later
  incoming matter acquires no earlier owners (845);
- reservation needs explicitly selected owned lots summing exactly to the
  recipe, and current craft eligibility (843, 845, 838);
- reserve, three work steps and output count every milligram once, and the
  input claims move to the output (837);
- the input owners grant together, revocation is a new proposal, and the
  maker gains nothing by reserving (839);
- an eligible authorized subject resumes another's work (838);
- interrupted work stays at its creation site; carrying is chosen,
  follows the carrier, refuses a carried final step without an output
  custody relation, and rests where dropped (840);
- an owned debit refuses without selection, takes an exact portion of a
  named lot and leaves the rest (846, 849);
- consumption refuses without an authored disposition or with an
  unallocated portion; `CloseConsumed` and `Retain` close or retain each
  portion (850);
- the planted faults are caught and restored.

### G. The society feature (886, 893, 894) — landed

`shared/isocosm` has a `society` feature, default on. It gates operations and
never state: `isocosm::SOCIETY` and `isocosm::society()` refuse, in a build
without the feature, the commands `Party`, `Project`, `Goal`, `Institution`,
`Custody`, `ActOwned` and `Owned`; the `StockOwnership` and `SpreadOwnership`
assertions; the effects `ReserveProject`, `WorkProject`, `ProjectDecision`,
`InstitutionDecision` and `Institution`; their queries; goal directives; the
owned quotes; and `polity_reading`. Mesocosm opts out with
`default-features = false`; the VTT, Eponym and `host` keep the default.
Tests in `tests/society_gate.rs` run in both builds. Done when:

- a build with `--no-default-features` refuses every society command with the
  feature's own message and leaves the state hash and project records
  unchanged (commands, assertion, effects, queries, directives);
- the same inputs reach the layer, and are refused for their own reasons,
  with the feature;
- a state carrying society fields round-trips untouched in both builds, and
  unused fields stay out of the save;
- Mesocosm's dependency graph resolves with no `society`, and the VTT,
  Eponym and `host` with it;
- the gate faults are caught by the featureless run.

### T2. Item custody (817, 820 to 823) — landed

`tests/society_custody.rs` and `tests/society_custody/`. Journey semantics
(817, 820 to 823) already have unit tests in `src/journey/tests.rs` and
`location_tests.rs`; this phase adds custody over them. Done when:

- a carried inert item rides its carrier's one journey and rests where
  dropped, with mass conserved and replay identical;
- custody refuses without native control, contact, a provider or a free
  item, and burden counts items and carried projects against capacity;
- titled stock keeps its owners through carry, journey and drop, and an
  equip spending titled stock needs an exact selection and a disposition;
- the faults are caught.

### T3. Party and disclosed maps (853, 854, 857, 858) — partly landed

`tests/society_party.rs` and `tests/society_party/`: only the played body
opts in, withdrawal after a body change, disclosures bounded by native
knowledge, and records that outlive membership and replay identically.
Open: `ObserveStatement`, `Tell` and `ObserveTerrain`, and the shared map's
rich `DiscloseDetails` records. Done when each refuses a body that does not
hold its source, and records only what its member natively holds.

### T4. Institutions, polities and goals (844, 866, 867) — landed

`tests/society_institutions.rs`, `tests/society_goals.rs`. Done when:

- a constitutional operation is proposed by a member, decided by its
  authored electorate, and applied once by its executor, with support
  required and a titled debit needing an exact selection;
- an amendment cannot rewrite the founding cause, method, account, host or
  name a participant, and dissolution ends the polity by its event;
- a polity reading keeps authored attributes apart from member facts;
- an accepted act records participation in every goal it served, an ended
  goal takes none, and explicit records bind to the actor's own event;
- the faults are caught.

### T5. Polity-owned stock (839, 844) — landed

`tests/society_collective.rs`. Done when each authored strategy grants only
as authored (unanimous members, declared representatives, a governance
decision through its process with a witnessing act), an unavailable member
blocks unanimity, and a collective owner must name a valid, unambiguous
strategy; the faults are caught.

### T6. Spread titles — placement landed

`tests/society_spread.rs`: owned site stock placed into a native identity
needs an exact selection and an authored `SpreadPlace` disposition, keeps its
owners as the identity's claim, closes the portion its disposition closes,
and refuses a missing or inconsistent disposition, with the unselected
portion left titled. Overflow delivery is C11's own test. Open: fork strain
outputs (`SpreadOutput::GeneratedStrain`) and produced spread references
(863). Done when each retains or closes exact claims with conserved matter
and replay, ambiguous or spent references refuse, and the faults are caught.

### T7. Consumers of a debit selection — open

Direct owned commands (`Command::Owned`: wound, member edit) and graft carry
selections. Institutions and custody are covered by T2 and T4, and spread
placement by T6. A founded body holds no tissue in its parts, so a wound
spills nothing to debit; this phase needs a body fixture with tissue first.
Done when each consumer refuses an unselected owned debit and admits an exact
selection with its authored disposition, and the faults are caught.

### Not in this plan

Table time and downtime consent (851, 852, 855, 856) live in the VTT's
`crates/isometry-views/src/downtime.rs`; pace (860) is native sim code
outside the society layer. Neither is tested here.

## Findings

- 2026-10-14: an actor sets aside unfinished work by selecting no project
  (`Change::Act` with `project: None` clears its binding in
  `projects::admission::project_candidate`) and may then reserve another.
  The first project keeps its reservation, progress and lots (840); the
  reservation gate in `projects::readiness::project_ready` refuses only
  while the *selected* work is unfinished.
- 2026-10-14: an owned `Effect::Transfer` keeps its selected claim without
  any authored disposition: `stage::staged::owned::owned_transfer` sets the
  destination as the preserved slot, and `debit::disposition::finish` turns
  each preserved piece into a new claim at that holder. `Effect::Spend`
  preserves nothing and needs an authored disposition (850).
- 2026-10-14: the owned path engages for every process once any lot is
  asserted (`debit::runner::begin_process_debits`), so an unrelated plain
  `Command::Act` that debits an owned account is refused, not allocated.
- 2026-10-14: the replay witness hashes `Projects` by serde
  (`simulation::witness`), so a project field skipped by serde would let a
  replay drift unnoticed. T1's carry test asserts custody changes the hash.
- 2026-10-14: the `society` feature is a runtime refusal at each entry
  point (`cfg!`), not a compile-out: the society code stays compiled in every
  build so `State` keeps its types. A compile-out would need the types split
  from the operations in `projects`, `institutions` and `polities`.
- 2026-10-14: an operation's ordinal counts staged effects in order, so in a
  process whose first effect is `Relate`, a following `Spend` is operation 1
  and its authored `OperationRule::Process` needs `ordinal: 1`.
- 2026-10-14: an embodied body keeps its matter in its parts, not its
  accounts; a founded critter holds none in parts until embodied, and
  `Embody` through a command does not move accounts. A test body that must
  pay stock is embodied at founding with the matter in its part.
- 2026-10-14: a body "knows" a site only after staying a tick there
  (`visits` records only `arrived < tick`), so a disclosure of a visited
  site needs `advance(1)` before the departure.
- 2026-10-14: the existing C11 owned-quantity test cannot run in a build
  without the feature and is gated on it.

- 2026-10-14: the source enforces six guards twice, so no single-span fault
  can change behaviour and none is planted: custody's contact and capacity
  (admission and `check_custody_changes`), a repeated vote and a repeated
  application (admission and the `Records` store), a collective owner's
  ambiguous strategy (`check_project_lots` and `reserve_project`), and a
  placement's owned-account guard (`placement_relevant` and `guard_debit_at`).
  The tests still assert each refusal. Proving one layer alone would take a
  multi-span fault.

## Progress

- 2026-10-14: plan drafted (882's stock/title boundary). T1 landed: 13
  tests; the first 15 faults were each applied in a detached worktree, failed
  their named test and restored byte-exact.
- 2026-10-14: after 893 and 894 the plan covers the whole society layer. The
  gate and T2 to T5 landed with 32 further tests in the new files; the
  featureless build runs `society_gate` only.
- 2026-10-14: with the gate, `cargo test --no-default-features` passes 602
  tests of 603 in 48 targets (the one failure was the C11 owned-quantity test,
  since gated on the feature and passing in both builds). `cargo check
  --workspace` passes for the VTT root, Eponym and `host` with the feature,
  and for the whole Mesocosm workspace without it.
- 2026-10-14: the full fault run applied 53 faults one at a time in a
  detached worktree; 48 failed their named test and restored byte-exact. Five
  on double-enforced guards were not caught (two of them also restored
  wrongly, and a third fault's replacement was not unique in its file), and
  they were dropped; every replacement must now restore uniquely (`restorable`
  in `controls.py describe`). The nine faults on files the failed restores
  had left changed, plus the four spread faults, were rerun on a clean tree:
  eight caught and restored, and a sixth double-enforced guard
  (`placement-unguarded`) dropped. 51 faults remain.
