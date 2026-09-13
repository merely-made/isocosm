# Paredros functional loops and wiring plan

**Status: in progress, 2026-09-13.** J0 body inspection, J1a controlled-session
persistence, B1 charged limb contributions and B2 cardinal strike adjudication
are implemented, along with B3 treatment and J1b fractional terrain motion.
The native client uses the same movement, injury and inventory owners. Broader
contact mechanics, construction and full adventure coordination remain open.

## Direction

Build systems that produce situations. A curated encounter is optional content,
not the prerequisite for developing material life, bodies, memory, or world
conditions. Existing F0-F8 milestones in the
[execution plan](2026-08-07_paredros_execution_plan.md) retain their semantic
done-conditions. This plan owns the next implementation dependencies across
them; the damaged crossing remains a reusable contact fixture.

The user requested five connected guarantees: injury affects the played body,
actions, attachments and inspection together; strange mechanics follow meaningful
world conditions; terrain changes reach creatures and rendering; saves preserve
consequences and continuation; and creatures answer from their own knowledge and
preferences. Hagiograph is a named lane alongside ordinary individual memory.

The current material and social vocabularies are intentionally small. Extend
them through actual operations rather than adding a catalog before its verbs.

## Existing owners and gaps

| Owner | Current implementation | Join still needed |
| --- | --- | --- |
| `paredros-world::GameState` | Coordinates world, movement, bodies, admitted anatomy, items, intents and events | Continuous contact effects and terrain work must enter this accepted history |
| `ContactWorld` | Fixed-step movement, board handling, attack/brace, integrity and grip/reach impairment; its own save | Bind runtime bodies to subjects and consume one durable body outcome |
| `EquipmentSession` | Authored subject, two dressings, attach/detach and restricted replay save | Safe stale/dead inspection first; host-selected subject and general session later |
| `Simulation` | Needs, navigation, population and autonomous actions over `GameState` | Controlled-subject scheduling and coordinated real work |
| `Projects` | Durable `Visit` goals | Material, repair, treatment and cooperation goals |
| `Society` / `EpistemicLog` | Agreements, deeds, observations, reports and corrections | Belief- and norm-supported answers; bounded recall |
| `Control` | Recorded begin/tag-in/tag-out/succeed pointer | Product validation of death, eligibility and outsider generation |
| `Sortie` | Older composed travel/wound/salvage/control receipt | Reuse its behavioral evidence; retire duplicate authority as the joined session adopts it |
| Mere / Mesocosm | Nisus voxel edits, Modulus traversal, Conatus spatial mechanics, body documents and generation | Product bindings and acceptance, not a new shared game evaluator |

Current Paredros dependencies pin a Mere revision. Inspect that pin's API before
adopting newer local Nisus/Modulus/Conatus work; a local implementation elsewhere
is not proof that Paredros's pinned dependency supplies it.

## Lane map

| Lane | Feature target | First useful completion | Dependencies |
| --- | --- | --- | --- |
| J: session and consequences | One played subject in the persistent simulation | Contact injury, inventory and inspection agree across reload | Existing world/body owners |
| B: body and directional action | Damage, treatment and limb-dependent attacks/defenses | Injury removes or changes a specific legal action and allows an alternative | J for playable join; pure rules can precede it |
| W: world conditions | Surgical risk plus independently composable magical laws | A conditional body operation and a stored/transmitted effect have explainable costs and counters | B for surgery; J commits; magic can precede surgery |
| T: materials and construction | Gather, carry, store, place, dig, repair | A paid-for terrain edit changes traversal and visible geometry together | J ordering; existing spatial/voxel mechanics |
| M: memory and judgment | Individual recall, preferences, beliefs and answers | An absent creature learns a report and changes its response | Accepted events; existing social owners |
| H: Hagiograph | Retelling, remembrance, significance and manifestation proposals | Remembered history changes a later interaction or admitted world proposal | M plus W for material manifestations |
| S: save and continuation | Coherent snapshots, resume, death and succession | Save/reload continues the same consequences through another life | Incremental requirement on every lane, not a last phase |

Detailed law design lives in [world conditions](2026-09-09_world_conditions_plan.md).
Individual memory and Hagiograph live in
[memory and remembrance](2026-09-09_memory_and_remembrance_plan.md). These are
Paredros consumer plans; any promoted shared contract needs its own owning-repo
review and a second consumer. Wing architecture remains in
`mesocosm/design_docs/2026-07-30_games_wing_founding.md`.

## J: join the existing owners

### J0. Safe inspection

Preserve the authored equipment fixture while making historical/stale anatomy
and dead subjects inspectable. Query the existing accepted state; the UI must not
repair anatomy, synthesize capabilities, or revive a subject. A stale detailed
body must be visibly distinguished from current anatomy. Released equipment
remains at its accepted location, not silently reattached by a redraw.

Done when focused tests inspect current, injured/stale, reconciled/severed and
dead states without panic or mutation, with unavailable operations explained.
This is a prerequisite only: it does not widen the equipment save whitelist or
connect the crossing to the played subject.

### J1. A product session

**J1a, first bounded implementation:** compose existing `GameState` and
`Control` without exposing a second mutable game. Admit one living named subject,
dispatch only that subject's actions, and permit continuation to another existing
living named subject only after the controlled life dies. Save control decisions
at game-intent cuts and validate them against the state at those cuts on replay.
A once-valid successor may be dead by the final save; a currently dead subject
awaiting continuation is also a valid session state. Hash agreement cannot
replace these semantic checks. Reject post-Begin actions for another subject.

This bounded session has no autonomous scheduling, society, contact binding,
outsider generation, general host Save/Load integration, or checkpoint compaction.
It is the foundation for the wider J1/S1 contract below. Existing-life selection
here validates life state only; social eligibility and the outsider arrival policy
remain product rules to add before claiming the full F8 behavior.

Introduce a small product session coordinator, as a module before a new crate.
It composes `Simulation`/`GameState`, controlled subject, social state and an
explicit runtime binding from contact `BodyId` to `SubjectId` and body revision.
The session owns accepted command order and clock conversion; it does not copy
the state owned by its components. Autonomous policy must not issue a second
action for the controlled subject while the player drives it.

Define which contact positions/velocities must survive interruption and which
runtime objects can be rebuilt. Existing integer movement and continuous contact
coordinates need explicit units and conversion; truncating each frame into a
voxel is not a valid authority bridge. Distinguish fixed simulation steps from
accepted intent sequence numbers and world time.

Contact detects an impact; product rules admit a body consequence once, then
contact capabilities and the sheet derive from that revision. Runtime integrity
must not independently decide a different lasting injury. Reject stale subject
bindings before applying effects. Save/load uses S1's common cut.

Done when one existing named subject moves through contact, suffers a fall,
loses capability, and is inspected with the same identity, revision and item
locations; duplicate effects cannot injure twice; a reload continues identically.

## B: bodies, injury and directional action

### B1. Damage and recovery without requiring combat

Begin with a fall, crushing work accident or environmental exposure. Record
affected part addresses, cause, load/exposure, wound, and resulting revision.
Distinguish temporary impairment, tissue damage and severance. Loss follows
the admitted part tree. Treat attachments, detached matter and equipment
custody atomically with the body outcome. Recovery restores only what its law
permits; resting cannot silently grow back a missing limb.

Done when damage changes movement/work/action eligibility and equipment together;
treatment consumes an available resource and has a recorded result; descendants
of a severed branch cannot remain usable; player and autonomous subjects obey
the same rules. Impossible treatment changes nothing.

### B2. Directional combat

Adopt the requested directional attack/defense shape as a Paredros action profile.
Input chooses an intended direction relative to facing; anatomy and equipment
determine realizable trajectories. Start with a bounded set of authored arcs
(left, right, overhead, thrust), then support body-specific alternatives without
assuming every creature has two hands. Grip, reach, joint movement, occupied
parts, balance and the held implement constrain an arc. Alternative limb bindings
can preserve an action at a different cost or reach after injury.

**2026-09-09 refinement from Mark:** directions, swings and effects vary with
anatomy. Extra arms can prepare additional punches during a swing. Represent
this as a coordinated action with explicit participating part addresses and
per-limb preparation/release/recovery, rather than multiplying damage by arm
count. The player can reserve an available arm for guarding, carrying or a later
strike instead. A tail, tentacle, jaw or articulated tool can supply different
arcs and effects; the input arrangement follows reachable actions.

Additional strikes share balance, energy, footing, attention and target access.
Each has its own contact and outcome; a single successful roll cannot make every
prepared fist land. Recheck limbs at preparation, release and contact. Loss or
occupation of one arm cancels or changes only dependent subactions, with explicit
committed costs and recovery. Extra limbs expand coordination choices while
retaining interruption, congestion and resource costs. Their useful count is
bounded by admitted anatomy and concurrent-action limits, not a fixed two-arm UI.

Use explicit wind-up, committed action, contact and recovery phases. Revalidate
the binding if a part fails between preparation and contact. Input assistance,
direction selection and difficulty are settings; deterministic rules record the
accepted choice. Guarding depends on coverage and functioning bindings, not just
matching an animation label. Physics supplies contacts, product rules resolve
injury. Initial geometric envelopes need not wait for full articulated animation.

Done when two different body arrangements expose different valid arcs; loss of
one binding changes offense and defense mid-action without granting a free hit;
range, facing and timing matter; the same accepted input stream replays. Headed
acceptance checks direction readability and input comfort, without prescribing
a story, location or camera as the game's permanent form.

### B3. Strike quality and stochastic resolution: investigate before tuning

Mark proposed combining the physical swing with a roll weighted by strike
quality. The following source review supports trying this as a Paredros-native
combat policy; it does not establish exact coefficients or a RAW adaptation.

| Primary source, checked 2026-09-09 | Observed design | Paredros implication |
| --- | --- | --- |
| [TaleWorlds weapon model](https://www.taleworlds.com/en/Games/Bannerlord/Blog/32), 2017 developer account | Contrasts earlier random damage ranges with weapon length, mass, inertia and a simplified body-driven swing model | Derive contact quality from the participating body and implement, rather than a generic damage randomizer; this is historical design evidence, not a current-version benchmark |
| [OpenMW 0.49 release account](https://openmw.org/2025/openmw-0-49-0-released/), 2025 | Documents moving initial attack evaluation to release for animation/audio fidelity, with a range check still at impact | Preparation/release/contact are distinct; record exactly when chance is sampled and revalidate geometry before applying its result |
| [N'Garde author description](https://www.nexusmods.com/morrowind/mods/58658), checked 2026-09-09 | Offers glancing blows while preserving the underlying hit-chance mechanism, plus active defense | A failed roll can have a legible weak/deflected contact outcome rather than an apparently solid strike passing through a body |

These sources are design references only; no source code or game assets are
copied. The N'Garde option changes outcome behavior; preserving a probability
formula does not mean preserving the original game's complete rules.

Compare three policies against the same recorded contact cases:

1. **Contact-driven:** geometry and material response determine the outcome;
   skill changes preparation, control and recovery. Establish this baseline.
2. **Quality-weighted outcomes (recommended experiment):** valid contact,
   alignment, timing, speed, leverage, guard coverage and footing determine a
   quality record. Character technique and state shape a bounded distribution
   over glancing, effective and exceptional consequences.
3. **Attack-roll adapter:** use an existing rules procedure and its legal inputs.
   Physical input chooses legal attempts/targets or presentation. Giving a
   better swing an unauthorized roll modifier is an explicit adaptation, not
   rules-as-written fidelity.

Keep contact existence, guard interception, armor response and injury severity
separate. A roll cannot produce contact through a wall or with a severed source
part. Avoid double-counting speed/skill once in geometry and again as unrelated
bonuses. Quality weighting is a native policy over world facts; adapting the
mechanics does not require rewriting the world's material or anatomical history.
If another rules profile cannot represent those facts, retain them as annotated
facts or refuse the operation rather than replacing them silently.

Sample admitted uncertain outcomes under a saved RNG algorithm/version and
action/subaction identity. UI previews never consume/retry the authoritative
draw. Record rule revision, quality inputs, committed costs, draw identity and
outcome. Reload, a changed frame rate and sibling-limb iteration order must not
reroll the same admitted attack. Explicitly selected alternate stochasticity
settings are saved combat-policy revisions.

Done when contact cases cover empty swings, poor alignment, guarded/glancing
hits, good placement, disabled limbs, interrupted charged combinations and
different body arrangements. Fixed-input trials compare distributions,
monotonic response to improved quality (holding other facts equal), defender
agency and exact replay. Headed play checks whether players can tell why an
outcome happened. Accept a weighting policy only after that comparison; do not
make the first model's weights a permanent world law.

## T: materials, construction and work

### T1. Material transactions

Extend `Items` beyond Food/Dressing/Scrap as operations demand: quantities,
material identity, condition and location/custody. Add drop, transfer and storage
before crafting an expansive catalog. A work command names a subject, tools/body
capabilities, source material, target region, expected revisions and work cost.
Gathering, placing and repair move or consume material exactly once. Refusal
must leave inventory and terrain unchanged.

Done when one creature gathers, carries and places material with provenance,
cannot double-spend it, and can resume interrupted work. An autonomous creature
uses the same command boundary, with its own needs and agreements.

### T2. One edit, all spatial consumers

Accepted Ground edits produce revisioned dirty regions. Paredros feeds those
regions to the pinned voxel/spatial packages and its renderer adapter. Navigation,
contact, ray queries and rendering identify the source revision they represent.
If collider realization cannot complete, pause/refuse dependent simulation at
that revision; do not allow walking through a visually completed wall. A stale
render product is explicitly pending, not another editable world.

Done when adding a support permits a crossing and removing it changes collision,
navigation and rendering from the same edit; an occupied-cell edit has a defined
refusal/displacement policy; negative coordinates and chunk borders work; reload
reconstructs both consumers from accepted terrain without persisting GPU handles.

### T3. Work and consequences

Extend `ProjectGoal` from visits to gathering, placing, repairing and tending.
Agents evaluate affordability and capability, negotiate help and retain partial
progress. Structures retain builder/material references and useful effects such
as support, cover or storage before adding a full settlement economy. Changes
can affect another creature's access, safety or possessions, feeding M's observed
events and personal judgments.

Done when work can be interrupted and resumed by an eligible subject, refusal
is a complete outcome, and another creature can respond to an observed loss or
benefit without requiring a scripted encounter.

## S: saving, death and continuation

### S1. Coherent session persistence

The equipment store remains a restricted fixture. Define a versioned adventure
envelope naming world/generator/rule revisions, accepted event cut, component
records, control history, and required content identities. Save clocks, random
state, pending work and in-flight actions explicitly or restrict saves to a
documented stable boundary. Do not independently snapshot mutually dependent
owners at different ticks.

Restore into a candidate session, validate references and component hashes, and
rebuild projections before replacing the active session. Corrupt/missing content,
unsupported versions and incomplete archives must preserve the running state.
Use immutable publication already demonstrated by the equipment store as a
mechanism, not its fixed-subject admission policy.

The [long-lived save strategy](2026-09-09_memory_and_remembrance_plan.md#long-lived-save-strategy)
owns checkpoint/replay floors, recent tails, semantic retention and size receipts.
Saving an ever-growing copy of every intent remains an initial correctness
format, not the long-term world format. Mark accepts reasonable growth with
world history and created content; years of repetitive updates must not make the
world impractical to save or restore.

Done when interrupted work, injury, terrain, agreements, recall and control resume
from the same cut; the next accepted actions match uninterrupted play; injected
partial/corrupt saves never partially replace live owners.

### S2. Death and continuation

The product session checks death and successor eligibility before using
`Control::Succeed`; `Control` itself does not validate living-world facts. Preserve
the previous body's remains, item custody, commitments and others' recollections.
Use an eligible existing life or generate an outsider when no such life exists.
Record generation and selection. Optional non-death control changes remain
settings or admitted world mechanisms with their own consequences.

Done when both continuation paths work and survive reload, a dead subject cannot
act, and changing the control pointer neither transfers property implicitly nor
rewrites what other creatures remember. Hagiograph can remember the previous life
without requiring it to have become legendary.

## Orchestration and acceptance

1. Review the existing body/equipment WIP independently before committing it.
   Preserve its recorded 107-test/two-process receipt without claiming new tests
   inherit it. The current work does not absorb that patch into an unrelated commit.
2. Implement J0. In parallel, finish the W and M/H designs against existing code.
3. Settle J1/S1 command ordering, clocks, bindings and save envelope together.
   Terra owns that integration; Luna can implement bounded fixtures and rejection
   cases once the interface is written. Avoid concurrent edits to `state.rs`.
4. B1 and T1 can then use separate modules with serialized integration into
   accepted transitions. M1 can proceed independently over accepted deed evidence.
5. Add B2/B3, T2 and H retelling as their inputs exist. W's surgical operations
   consume B's revision boundary; W's first magic operation can proceed
   independently of surgery. H material manifestations also require W.
6. Validate connected sandbox loops using real production commands, varied
   subjects and changed conditions. A fixed script is a regression receipt,
   never the only supported way to play.

Every implementation reports exact files, supported operations, focused checks,
and remaining joins. Broader wing contracts are promoted only after another
consumer challenges identity/authority semantics. Reusable mechanical code may
enter its settled shared owner earlier.

## Findings

- **2026-09-09, pre-J0 inspection:** `EquipmentSession::sheet` assumed current
  living anatomy with `expect`. J0 removes that assumption; its loader still
  deliberately accepts only fixed fixture equipment history.
- **2026-09-09:** `contact.rs` owns `BodyState::integrity`, `Impairment` and
  `ContactSave` separately from `state.rs`; `crossing.rs::new_world` creates its
  own runtime subjects. Joining a widget does not close this authority gap.
- **2026-09-09:** `projects.rs::ProjectGoal` contains only `Visit`; `world.rs`
  exposes carving and site inheritance but not embodied building transactions.
- **2026-09-09:** `simulation_record.rs` already validates game/population/project
  replay; `control.rs` already replays subject changes. Neither is a whole
  adventure snapshot with contact and society.
- **2026-09-09:** local `nisus` and `modulus` provide shared mechanics, while
  `paredros-world/Cargo.toml` pins Conatus to `d82afa17`. Adoption must respect
  the actual dependency boundary.

## Progress

- **2026-09-09, follow-up:** incorporated anatomy-shaped coordinated strikes,
  independent skill/risk surgery, a proposed charge/sympathetic-coupling magic
  front, and a three-source combat-mechanics comparison. Long-lived persistence
  now plans independently restorable checkpoints and explicit replay floors.
  Terra implemented J1a with historical control validation; it does not
  close the wider contact, autonomous or social session joins.
- **2026-09-09, J1a implementation:** `paredros-world::Session` owns one
  `GameState` and a `Control` record. Its public API admits a living named
  subject, gates controlled actions, and permits existing-life succession only
  after death. `SessionSave` version 1 replays control at historical game-intent
  cuts, so later death of an earlier successor is valid; mismatched actors,
  ineligible subjects and invalid cuts remain rejected even with recomputed
  checksums. `SessionLimits` supplies caller-configurable byte and record-count
  bounds; writer and reader enforce the same archive boundaries. Byte limits
  bound encoded input/output, not all possible allocations during regeneration.
  The session has no host UI, autonomous scheduler, social eligibility, outsider
  arrival, contact bindings or compaction yet.
- **2026-09-09, J1a verification:**
  `cargo test -p paredros-world --lib --test session_boundaries --locked --offline
  -j 2 --target-dir target-contact` passed **35 library + 3 integration tests**,
  including 8 new session unit cases and 3 independent public-API boundary cases.
  It checks real item custody through death/succession/reload, dead awaiting
  continuation, repeated succession, same-state refusal, malformed historical
  cuts and exact/over-limit archives. The existing private cache is
  `C:/Users/mark_/Code/cargo-homes/paredros-save-check-20260908`.
  A concurrent Mesocosm matter receipt module lacked `StockError` in its import;
  the one-line `super::{Stock, StockError}` repair unblocked this dependency build
  without changing its behavior. No broader matter gate is claimed.
  The separate `cargo test -p paredros-room --lib body_sheet --locked --offline
  -j 2 --target-dir target-contact` regression passed **24 tests** on the updated
  dependency tree. Formatting and diff checks pass; changes remain uncommitted.
- **2026-09-09, save growth baseline:** the new
  `crates/paredros-world/examples/save_growth.rs` checks restored equality and
  unchanged physical world/body/items after repeated equipment cycles. Four
  checkpoints (0/100/1,000/10,000 pairs) produced 407/1,485/12,284/123,905 encoded
  bytes. The exact CSV is `testing/save_growth/2026-09-09.csv`; analysis and
  limitations live in the memory plan's long-lived save strategy. This is a
  single-operation history baseline, not a multi-year capacity claim.

- **2026-09-09:** Created lane/dependency plan from live code and the user's
  functional-loop direction. Terra implemented J0; Luna drafted M/H and
  Terra drafted W, with root review of causal facts, memory budgets and RAW scope.
  Further implementation is staged above, not reported done.
- **2026-09-09:** J0's `equipment_session.rs` reads the admitted historical
  anatomy and marks stale/dead state without repairing it; `equipment_view.rs`
  exposes the reasons. Four added tests cover stale attach refusal, explicit
  reconciliation/severance with released dressing, and lethal-fall inspection.
  The three state cases check that inspection leaves the authoritative hash
  unchanged; the view case checks that wrapped stale and dead reasons both remain visible.
  `cargo test -p paredros-room --lib body_sheet --locked --offline -j 2
  --target-dir target-contact` passed 24 tests using the existing private
  `cargo-homes/paredros-save-check-20260908` cache. This is automated projection
  coverage; injury input, general save admission and crossing integration remain
  open. No new native screenshot or physical-input receipt is claimed.

## B1: timed limb contributions (2026-09-09)

**Status: implemented; automated native verification completed.** Authorized after the shared functional evaluator and
wire-format commit `dadbd0c`. This slice turns finite-charge operation receipts
into a product-owned action lifecycle; it does not claim full geometric combat.

### Ownership and rules

A timed-action coordinator owns one existing Session, functional network and
current action. The Session retains sole GameState/body/inventory authority.
Contributions bind real part IDs of the controlled subject, have a direction
and accumulate committed charge over explicit integer ticks. Additional limbs
may join an existing preparation. Release emits one typed strike per surviving
contribution; it does not debit the committed charge again.

Part loss cancels only affected contributions. An explicit shared dependency
such as a gate may interrupt several. Charge already spent on an interrupted
contribution is not refunded. Resource exhaustion must be distinguishable from
limb loss. Save and restore preserve charge, timing, bindings and interruption
state alongside the same authoritative Session; restoring is not regeneration.

Injury currently advances body revision before detailed anatomy reconciliation.
The host must batch the accepted injury and reconciliation atomically before
reevaluating bindings. An unresolved stale anatomy cannot authorize a strike,
but does not identify which specific limb was lost.

### Lanes and done-conditions

- Terra owns the deterministic lifecycle, validated save envelope and tests.
  Done when two contributions charge, one real limb is severed through game
  transitions, surviving strikes release, and save/resume agrees with uninterrupted
  continuation. Invalid inputs and invalid archives leave the current state intact.
- Luna owns native input and visible direction/contribution status using this
  same coordinator. Done when mapped prepare/charge/join/release/injury/save/load
  controls exercise the model and their dispatch tests pass; headed/physical
  observations are reported separately from compile and automated dispatch.
- Root owns integration review, manifests, documentation and publication.
- The independent ecology lane finishes the core suite, including flow tests.
  Its result does not certify native action input.

### Remaining boundaries

ContactWorld-to-GameState collision outcomes, hit quality, damage allocation,
full equipment tool bindings, native pointer trajectory sampling and general
spell/enchantment execution need further consumer work. Succession remains owned
by Session; the timed coordinator does not yet expose transfer to a successor. This first action model
produces direction/binding/charge receipts for those adjudicators to consume.

### Progress

- 2026-09-09: read Session, current_anatomy and injury/reconciliation ownership;
  split model and native input lanes. Existing source-identity audit script is
  unrelated work and remains untouched.
- 2026-09-09: core lifecycle landed in source with transactional charge/release
  and injury batches. A full `paredros-world` run passed 91 tests. After the final
  routing-hop bound and batch-rejection addition, both timed-action integration
  suites passed all 9 tests (7 admission, 2 lifecycle). Coverage includes separate
  limb contributions, shared gate interruption, exhausted supply, partial loss
  followed by save/restore/release, death, stale anatomy, forged bindings and
  rejected transitions. Native handler and presentation receipts are recorded below.

Run the native surface with `./scripts/wing.ps1 paredros run -p paredros-client
--bin timed_action`. Arrow keys prepare a direction; hold Space or left mouse
for charge, release to strike the target, WASD moves, J joins the second limb,
I applies self-injury for debugging,
and F5/F9 save/load. `PAREDROS_TIMED_ACTION_SAVE` selects the save path.
`PAREDROS_TIMED_ACTION_SMOKE=1` requests the asserted lifecycle and one presented
frame. These controls currently use an authored two-limb functional network.

This save envelope bounds input bytes and action contributors/ticks. Session
history still uses the existing replay log; these checks do not establish a
bounded multi-year world size or a history-compaction policy.

- 2026-09-09: recovered the original independent Mesocosm gate after an agent
  interruption: `mesocosm-core` completed with exit 0, 670 passed and 1 ignored.
  The long-run matter conservation case passed (2,534.06 seconds for its suite).
  This closes that run, not verification of later composition commit `d10c259`:
  concurrent source edits prevented an exact compile-input attribution. One
  existing `unused_mut` warning was reported at `organism/kingdom.rs:333`.

- 2026-09-09: native `timed_action` built with locked Mere `fc382ac4` and
  Netrender `c77b0be8` pins. The bounded smoke passed preparation, joining two
  independent limbs, charging, atomic injury, exact save/load restoration and
  one surviving strike receipt, then captured a nontrivial frame and presented
  the window. The capture was visually inspected: both control lines and final
  release status were readable. It is an automated headed receipt, not physical
  keyboard/mouse acceptance. The host waits while idle instead of polling.
  Local artifacts: `C:/Users/mark_/Code/.tmp/paredros-timed-action-20260909-pinned/`
  (`smoke.png`, `smoke.save`, `stdout.log`, `stderr.log`).

- 2026-09-09: `cargo test -p paredros-room --bin timed_action --offline --locked
  -j 2 --target-dir target-contact` passed its one handler lifecycle test on the
  same Mere `fc382ac4` / Netrender `c77b0be8` graph as the headed smoke. Existing
  room dead-code and unused Vello-patch warnings remain unrelated to this slice.


## B2: strikes and embodied consequences (2026-09-13)

**Status: complete for the bounded B2 slice; automated gates pass.** B1 now causes actual target
injury. The initiating subject and the affected target are distinct identities:
Session authorizes the initiator, while GameState owns both bodies, positions,
anatomy and equipment. A released volley is one atomic recorded transition.

The first adjudicator derives bounded directional contact from admitted part
geometry and authoritative positions, with configurable reach, quality, harm and
severance rules. It records misses as well as hits. It does not claim SRD RAW,
continuous pointer-driven swing trajectories, or the complete ContactWorld join.
The existing fixed-step contact probe retains its separate experimental status.

A hit changes the target's existing body condition and revision, reconciles
anatomy and releases attachments on lost parts at the same accepted cut. Death
must remain a valid consequence of a multi-limb volley. Bad targets, stale
bindings, malformed rules and rejected transitions preserve both action and
world state. Save/resume must preserve and continue the resulting consequences.

Lanes:

- Terra: research then implement the world-owned adjudicator, transition and
  timed-action integration, including replay and focused tests.
- Luna: audit the native crate's name, then wire the accepted API into native
  controls and target/consequence inspection after path ownership is settled.
- Root: naming/manifests, independent rejection and continuation review,
  integration, documentation and publication. Shared Mesocosm anatomy/rendering
  changes require coordination with their existing lanes.

Done when a released charged action can geometrically miss or injure a real
second subject; a severed part drops its attached equipment; subsequent action
availability and inspection agree with that anatomy; save/load preserves those
facts; and native controls exercise the same owner API. Capture/input evidence
is labelled separately from model and replay tests.

- 2026-09-13: renamed the native package/directory to `paredros-client`, its
  dominant input/rendering/inspection role. The `room` fixture and all existing
  binary/environment names remain. Historical receipts retain their original
  package name; current commands and imports follow the new package.

The bounded adjudicator uses each contributing part's placed bounding box,
extends it in one cardinal direction, and selects the first overlapping target
part along that direction (PartId breaks exact ties). Positive overlap is
required. Terrain visibility uses a ray between the source and target part
centres, so this is not swept-volume terrain collision. Root contact can injure
or kill, but cannot sever the root. The entire volley reads the same pre-hit
anatomy, then aggregates harm and reconciles severance once.

`CombatRules` is explicit input recorded with every volley. Revision 1 uses
`min(1 + charge / charge_per_reach, max_reach)` for reach; quality is the smallest
positive overlap dimension in world units. Harm is base harm plus charge plus
quality, saturating at the integer bound. These are configurable baseline
policies, not a calibrated simulation or an SRD to-hit formula. The timed-action
wrapper supplies already paid contributions and rejects raw volley injection
through its ordinary game-batch API. GameState accepts validated recorded
receipts; replay does not independently reconstruct charge-routing history.

GameSave v4 appends the combat vocabulary without changing the save field
layout. Genuine v3 histories remain readable; a v3 record containing a v4 combat
intent is rejected. The regression fixture
`crates/paredros-world/tests/fixtures/timed-action-v1-game-v3.save` is the verbatim
506-byte native smoke save produced on 2026-09-09 at `b9ae9a2` (SHA-256
`cf40be57866387048ae0237bcd0869c5249e882660735209b7a6089bbd091c87`).

- 2026-09-13: native handler tests pass movement, a fully charged wrong-direction
  miss, and actual target injury/severance/item release followed by exact
  post-hit save/load. The native fixture uses distinct positions and an authored
  raised rear limb to accommodate its sloping ground; strikes still use the
  ordinary admitted geometry and adjudicator. These are automated handler tests.
- 2026-09-13: aligned Paredros's `conatus`/`modulus` pins to Mesocosm's existing
  Mere `4f4de1d05ec99461f7fa3cdc4e514e904a999213`. The previous `fb7e136b` selection
  produced incompatible `BrickMap` types at the shared lens boundary. Netrender
  remains `c77b0be84fb6fc28a3c1602a2b1637f7d913acc0`; Parley remains Genet
  `3a7b50230d447f6fa7ed6921cba019f78347d932`. New receipts use these selections.
- 2026-09-13: full `paredros-world` suite passes **104 tests** on the aligned
  pins, including the real v3 archive, rejected-input atomicity, outer-surface
  contact ordering, lethal two-hit resolution and a survivor's available action
  becoming unavailable after severance, followed by restored continuation.
  `paredros-client --bin timed_action` passes **3 handler tests**. The runnable
  binary's bounded headed smoke also passes and its capture was inspected:
  target vitality 83, rear part 2 severed, item 3 dropped at `[-45, 13, -52]`,
  all preserved after save/load. Local artifacts:
  `C:/Users/mark_/Code/.tmp/paredros-combat-20260913-final/` (`smoke.png`,
  `smoke.save`, `stdout.log`, `stderr.log`). This is automated headed acceptance;
  physical keyboard/mouse testing remains open.

Reproduce from the umbrella with `./scripts/wing.ps1 paredros` followed by:

```text
test -p paredros-world --offline --locked --target-dir target-contact -j 2
test -p paredros-client --bin timed_action --offline --locked --target-dir target-contact -j 2
build -p paredros-client --bin timed_action --offline --locked --target-dir target-contact -j 2
check --workspace --all-features --all-targets --offline --locked --target-dir target-contact -j 2
```

These runs used `CARGO_HOME=C:/Users/mark_/.cargo`; the older isolated cache had
been retired. Ignored Paredros Cargo.lock SHA-256:
`7633c59574141afefb250b09fa13f7e8bae9f468fe9e7fb91b86b7e8f2e1b6dd`.
The client requires its selected target to exist when loading an archive; the
general v3 migration receipt belongs to the underlying session/game API.

The final workspace check passes every Paredros target and feature, including
Sortie and the retained room/residency/depth consumers of the renamed client.
The existing unused Vello-patch warning remains; the default native build also
reports the existing unused `retarget_from_ground` helper. Broader ContactWorld
integration, continuous swing trajectories and physical input acceptance remain
separate work.


### B3. Injury and treatment keep detailed anatomy current (2026-09-13)

`Session::fall(distance)` and `Session::rest()` compose the existing body-change
intent with anatomy reconciliation in one atomic caller operation. They require
current anatomy at entry. A surviving revision change records two ordered game
intents, with an empty new-severance list: refresh the revision while preserving
all existing lost parts. A safe fall or rest without healing records only the
ordinary intent. GameState retains its historical low-level transition behavior.
There is no new save vocabulary or hash layout; real v3/v4 histories remain valid.

Treatment uses the existing world's dressing and rest rules. One available
owned dressing is consumed when wounded; without one rest reduces fatigue but
does not heal. Treatment does not regrow a limb. A fatal fall leaves the last
admitted anatomy as historical evidence; current-anatomy queries reject the dead
body, and treatment cannot revive it. TimedActionSession wraps these operations
with contribution repair at the same atomic boundary: surviving charge remains,
lost or dead-body contributions are cancelled, and bindings follow the current
revision.

The native timed-action client exposes E to pick up a dressing at the player's
position and R to rest. The existing I command remains an explicitly authored
injury/severance debug action. The display reports the player's wound, vitality
and remaining dressings so treatment has a visible consequence.

Contact research identified a separate prerequisite for J1: Movement currently
owns integer ground positions, while ContactWorld owns fractional positions and
velocities plus independent fixture integrity/recovery. Adding continuous pose
fields changes serialized GameState hashes, including existing archive checks.
The next contact implementation must settle one position owner, units, clock
conversion and migration together. A disposable fall fixture would not settle
that boundary and is not the chosen integration strategy. B3 does not claim
continuous movement or a new collision-to-injury bridge.


B3 verification on 2026-09-13: **110 world tests** and **5 native timed-action
handler tests** pass on the B2 pins. The actual binary's automated window smoke
passes, including injury, dressing consumption, retained severance and exact
post-treatment save/load. Its reviewed capture shows player vitality 100,
wound 0, dressings 0, lost parts 1; target vitality 83 and its dropped item remain
unchanged by player treatment. Artifacts are in
`C:/Users/mark_/Code/.tmp/paredros-treatment-20260913/`; gate logs are
`.tmp/paredros-body-change-{world,native,build}.log` under `Code`. Reproduction
uses the same B2 test/build commands above. This is automated handler and headed
evidence; physical keyboard/mouse acceptance remains open. Independent Terra
review found no remaining atomicity or replay issue. Native actions were split
from `model.rs` to keep it below the 600-line ceiling.


### J1b. World-owned fractional movement (2026-09-13)

The first movement join uses `GameIntent::AdvanceMotion`, not an imported
ContactWorld save or host-submitted position. The input names a subject, current
body revision, next motion step and explicit movement rules. GameState checks
that binding, solves terrain contact, records the resulting pose and applies any
landing injury atomically. Existing timed-action admission repairs prepared limb
contributions after that same transition. There is no second integrity value.

Movement's precise pose is the latest accepted pose receipt in its own history.
Its old integer position map is a derived logical-cell cache: navigation, item
pickup/drop and the existing cardinal combat baseline still use that cell.
Fractional movement never starts from the truncated cell. Legacy integer Move
is rejected after a subject begins continuous movement, rather than silently
throwing away velocity or residual position. Autonomous scheduling has not yet
been switched to the new motion input. Combat remains cell-based until a
separate revision introduces precise strike-space semantics.

Coordinates are signed Q16.16 voxel units (65,536 quanta per voxel), with checked
i32 cell bounds. Position is the feet centre; genesis centres x/z in the logical
column and places feet at its integer y. Cell projection uses Euclidean floor,
including negative coordinates. Velocity is quanta per second. Each accepted
motion input advances exactly 1/60 simulation second and increments the subject's
motion-step count. Game-intent ticks remain accepted-command sequence numbers;
they are not seconds. MovementIntent ticks are ordinals in that subsystem's
own movement stream. The native host samples held WASD, admits at most eight
catch-up steps per wake, and pauses excess wall-clock debt. Input release and
focus loss clear held movement. Save/load restores the simulated pose and clock,
not the physical keyboard state.

The disposable Conatus character query receives nearby generated Ground voxels
in coordinates relative to the current cell, avoiding large absolute f32
positions. Results are quantized before the next step and checked against exact
Ground occupancy. Rules revision 1 provides bounded, configurable speed, gravity,
terminal velocity and a box stance shape. This is terrain contact for an explicit
stance shape, not a full anatomical collider, creature-to-creature collision,
step climbing, jumping, boarding, tethering or a migration of the crossing's
fixture mechanics. Grounded state, vertical velocity and fall-start height persist
midair. Landing converts accumulated height loss to completed whole voxels once;
only a harmful landing applies the existing fall rule and anatomy refresh.

GameSave writer v5 appends vocabulary without changing old enum tags or state
field layouts. v3 and v4 remain readable with their original expected hashes;
either may contain a motion intent. The real 801-byte v4 native treatment save
from B3 is retained as `tests/fixtures/timed-action-v1-game-v4.save` alongside the
existing v3 fixture (v4 fixture SHA-256
`1b99e8eeb99dedc55eea8e5194d755b94b7151a327684bec0a404fa549f80049`).
Both must restore and upgrade. Recomputed solver output is
covered by the saved state hash; matching continuation is tested on the current
pinned backend. Cross-platform floating-point replay has not been certified.

This is a correctness-first join: pose lookup scans accepted movement history,
and atomic admission clones candidate state. Saves still retain full command
history under configurable session limits. Checkpointing, archive growth and
long-session runtime cost remain open in the memory/retention lane; this slice
does not establish multi-year capacity.


J1b verification on 2026-09-13: **119 world tests**, **5 native handler tests**,
and the **all-features/all-targets Paredros workspace check** pass. The eight
session unit tests also pass after their mechanical extraction into
`session/tests.rs` to retain the 600-line ceiling. The landing regression starts
from generated relief and deepens the lower column through recorded World::Carve;
it verifies a single injury, current anatomy, no repeated injury while grounded,
and identical continuation after a midair save. The unmodified shallow ledge
produces only a safe landing under the completed-whole-voxel fall rule.

The actual native binary's automated window smoke passes and its capture was
reviewed. After load it displays precise position `[-56.433, 15.000, -51.500]`,
logical cell `[-57, 15, -52]`, motion step 1 and grounded state, alongside the
retained treatment/severance and target consequences. Artifacts:
`C:/Users/mark_/Code/.tmp/paredros-motion-20260913/`; gate logs are
`.tmp/paredros-motion-{world,native,workspace,session,build}.log` under `Code`.
Use the B2 reproduction commands above with the same pins and Cargo home.
Physical keyboard/mouse acceptance remains open. Existing unused Vello-patch
and default-build `retarget_from_ground` warnings remain.
