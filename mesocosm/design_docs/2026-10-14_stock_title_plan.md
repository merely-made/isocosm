# The stock/title plan: native tests for owned stock, projects and titles

**Date:** 2026-10-14

**Status, 2026-10-14:** in progress. T1 landed (owned stock and projects,
13 native tests, 15 planted faults caught and restored). T2 to T5 open.

Ruling 882 put titles outside C11: titled transit and owned-stock custody
are host and player commands, native-tested as a separate boundary owned by
this plan. C11's crowd keeps refusing owned stock, custody and titled
transit; nothing here reopens C11. Tuning is open (888) and the test
deferral of 826 is lifted for this surface.

The surface is the post-826 source in `shared/isocosm`: `src/projects/**`,
`src/spread/owned.rs`, and the consumers that take a
`projects::DebitSelection` or call `projects::project_control` (custody,
institutions, grafts, direct owned commands). At the 826 base `430d2e49`
it had one test,
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

### T1. Owned stock and projects (837 to 840, 843, 845, 846, 849, 850)

Tests in `tests/owned_stock.rs` and `tests/owned_stock/`:
`ownership.rs`, `work.rs`, `consent.rs`, `placement.rs`, `debit.rs`.

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

### T2. Custody and titled transit

`src/custody.rs` with `custody::Change` (carry, drop, equip, unequip with
selections), and journeys carrying owned stock. Done when custody keeps
native identity, stocks, owners and route authority across carry, drop and
arrival, refusals are byte-identical, and the phase's faults are caught.

### T3. Spread titles

`src/spread/owned.rs` beyond overflow delivery, which the C11 normative
test already covers: placement selectors (`SourceRule::Placement`), fork
strain outputs (`SpreadOutput::GeneratedStrain`), and produced spread
references (863). Done when each path retains or closes exact claims with
conserved matter and replay, and the phase's faults are caught.

### T4. Collective owners and produced references

Polity owners under each authored decision policy (844): `Governance`
through its Choice process, `UnanimousMembers`, `Representatives`; and
later processes selecting produced lots by operation/output reference
(863). Done when joint grants hold only with each owner's complete
electorate, stale constitutions refuse, ambiguous or spent references
refuse, and the phase's faults are caught.

### T5. Consumers of a debit selection

Direct owned commands (`Command::Owned`: wound, member edit, spread
overflow, fork strain, place), institutions' `Apply`, and graft carry
selections. Done when each consumer refuses an unselected owned debit and
admits an exact selection with its authored disposition, and the phase's
faults are caught. Institution and party semantics belong to their own
tests; this phase covers only their stock debits.

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

## Progress

- 2026-10-14: plan drafted (882's stock/title boundary). T1 landed:
  13 tests pass. All 15 compiled faults in `tests/owned_stock/controls.py`
  were applied one at a time in a detached worktree; each failed its named
  test and its exact bytes were restored (14 in `owned_stock`, 1 in
  `checkpoint11_spread`).
