# Functional networks, operators and generation

**Status, 2026-09-28:** Composition designed (404 to 407); no implementation lane open. *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status: first slice implemented, reviewed and committed, 2026-09-09.** Authorized first implementation slice
following the general model discussion sections 7.2 and 7.3.

**W1, 2026-09-18:** keep. Tier: sim. Index defect: filed below the Archive
heading. Evaluated against the wing design record; see
[2026-09-18_wing_plan_evaluations.md](2026-09-18_wing_plan_evaluations.md)
§2.

## Composition design, 2026-09-28

**Status: design refinement, not implemented.** Wing design record rulings
[404 to 407](2026-09-18_wing_design_plan.md#0-rulings-this-record-rests-on)
extend the intended model beyond the bounded evaluator whose receipt remains
below. Its three operators and charge routing are foundations, not the extent
of the effect vocabulary or a complete spellcasting model.

### Meaningful construction

Geometric and functional primitives compose into nested assemblies whose
internals remain addressable. A shared composition description should support
critters, plants, props, equipment and buildings; the rules give each domain
its meanings. Material and conceptual operations use this composition system.
Compatibility describes relationships and conditions rather than a list of
permitted finished designs. Constructible, executable in these circumstances,
and advantageous here are different questions. A disadvantage may become an
advantage as the world changes.

Construction declares functional properties. Geometry supplies required
measurements, such as reach, clearance, contact and coverage (405). The same
accepted construction and revision support functional readings, generated
form, colliders, animation bindings and descriptions. Structural attachment,
resource transport and control relationships need not be the same graph.
Motion can use reusable, stylized procedures that respect the construction.
No commitment to a new ECS, skeletal library or physics-based learning system
follows from this model. The
[body contract](2026-07-31_wing_phenotype_contract_plan.md#current-contract-refinement-2026-09-28)
owns identity and contextual embodiment; the sim plan owns capability,
repertoire and proficiency.

### Fundamental effects and equivalent exchange

Every fundamental effect admitted by a world's magic has a small cantrip
expression and can combine with others: burn, freeze, shock, buff, debuff and
heal are Mark's examples, not a closed enum. Cantrip describes a small
expression, not universal access, zero cost or a specific ruleset's casting
category. New fundamentals can be defined by an effect script; generation
can compose and vary admitted behavior, and modders can extend that behavior.

Each level of combining demands a magnitude more energy (404). The required
design is strongly escalating cost with deeper or stronger combinations;
the exact curve is open. The illustration of tenth-level magic combining
the strength of ten effects does not select tenfold growth, a numerical
definition of composition level, or a D&D/Pathfinder spell rank. Nor does it
say that ten sequential low-power casts are one tenth-level combination.

Explainable, surprising and powerful combinations are desired. Fairness means
the exchange and consequences must be understandable; it does not require
equally strong builds or removing every interaction that players exploit.
**Reading, not ruled:** costs should expose their contributing effects,
strengths, composition relationships and contextual modifiers. Composition
depth, repeated effects, duration, area, range, simultaneous versus sequential
use, and persistent propagation need explicit treatment before choosing a
formula. Equivalent exchange is a world rule, not a claim of thermodynamic
realism. Existing distinctions between ordinary magic and admitted divine
exceptions remain in the sim plan.

### Scripts connected to the world

An effect script defines its fundamental character alongside characteristics
that can vary with the sim. Possible world laws include nearby fire making
fire hotter and cheaper to cast, ice spreading until sunlight stops it, and
world size influencing gravity. These are admitted alternatives a world may
generate, not mandatory defaults. A script can depend on other generators'
outputs and on changing world conditions, so magic expresses the particular
world rather than only choosing names and colors.

**Proposed execution contract, not a selected scripting runtime:** an effect
definition identifies its revision, parameters, world inputs, targets,
operations, costs, ongoing behavior and explanation. It names what facts it
reads and may change. Reads use accepted world state and admitted generated
definitions; a query must not silently regenerate the rules of an inhabited
world. Mod-authored and sim-generated definitions pass through the same
admission and inspection path. The sim plan's current rule remains: lower to
process definitions where possible; hooks without a supported lowering stay
foreground-only until an appropriate background interpretation is designed.

Persistent effects need an explicit lifecycle: activation, due updates,
interruption and termination, including the event or condition that can stop
them. Reevaluate a dependency when it changes, and schedule ongoing work at
the mechanic's required grain. Self-amplifying interactions are not rejected
merely for being strong; their resource accounting, feedback and execution
cost must be inspectable. How to bound script work without silently changing
world outcomes remains an execution-design question, not a balancing rule.

### Rulesets and ways of casting

Categories such as schools or aspects are interpretations of the effects and
their relationships. Mark's references are Homestuck, Elder Scrolls, D&D,
Pathfinder and PbtA; these are influences to compare, not interchangeable
schemas. A ruleset's supplied spells can be distinctive exemplars of its
categories. Generated spells can fill out that catalogue and tailor it to
the world. Preserve each exemplar's defining behavior when specifying its
mapping; do not infer mechanical equivalence from a school label.

CoreRPG and the individual system adapters are unresolved. The same canonical
effect need not acquire the same spell level, numeric damage or casting
procedure under every ruleset. The overlay calibration requirements still
apply to shared world outcomes. Casting methods are another dimension and
remain open; this refinement does not select a universal mana pool, gesture,
preparation system, or progression model. Ball x Pit supplies Mark's analogy
for combinations, not an algorithm to reproduce.

### Design completion and inspection

Before selecting implementation slices, complete the description of effect
composition, context reads, resource exchange, ongoing actions and revision
changes across the construction, sim and overlay contracts. Record which
parts are authored, generated, derived once, invalidated by change, scheduled
or continuously resolved. An examiner should trace an action's support to
parts, equipment, procedures and conditions; show cost contributions and
refusal reasons; and connect accepted outcomes to their causes. Developer
explanations must remain distinct from what an inhabitant knows.

[Moirai](https://github.com/theor/Moirai/tree/d29de5dc1931965c73ae66a069e7c8fd51dc5df4)
was reviewed as comparison material on 2026-09-28: scheduled transitions,
property-gated reactions, and execution ancestry support efficient rule
authoring and inspection. This is source review, not a local benchmark or a
dependency choice. Isocosm's staged, checked effects remain the existing
authority boundary; an execution trace alone does not establish an actor's
motives or beliefs.

## Scope and ownership of the implemented first slice

One shared pure Rust evaluator lives in `shared/wing-functions/`. Its standalone
Cargo workspace preserves the three products' dependency resolution. It carries
part references, bounded networks, finite charge, typed operation requests and
receipts. It owns neither a second body nor product world state. Mesocosm supplies
current living part identities from `BodyDocument`; Isometry carries proposals
through its existing `GenValue` and generation-record machinery.

Sources and stores supply finite charge. Directed routes have capacities. Gates
and effect sites refer to parts. Strengthen and Project consume charge and return
typed effects for product adjudication; Store transfers it into a bounded store.
Range is caller-supplied, not a collision result. Units are authored abstract
charge units; this does not replace Mesocosm's conserved matter ledger.

## Lanes and done-conditions

1. **Functional evaluator (Terra):** deterministic allocation, bounded traversal,
   validation after load, atomic refusal and sequential operation batches.
   Done when missing parts, closed gates, bottlenecks, exhaustion and overflow
   have tested outcomes and errors leave state unchanged.
2. **Candidate generation and inspection (Luna):** parameterized deterministic
   networks bound to caller-supplied construction sites, for creatures and
   objects. Done when a seeded batch exposes outcomes and rejection reasons,
   and accepted values survive serialization without regeneration.
3. **Consumer wiring (root):** body membership adapter and typed proposal payload
   using the current generator carrier. Done when body loss changes evaluation
   and a proposal round-trips through a generation record's binary carrier.

## Findings

- 2026-09-09: existing `mesocosm-core::flow` records trophic matter movements,
  not internal functional connectivity. It remains the authority for that matter.
- Existing Isometry generator values already support objects and text. A
  versioned, validated construction payload can use that carrier without changing
  the public enum's binary variant ordering or introducing another Lua runtime.
- Concurrent edits in Mesocosm phenotype/graft, rules and world consumption are
  outside this slice and must remain intact.

## Open work after this slice

The composition design above is the broader target. The list below records
the first slice's outstanding integration and is not a closed vocabulary of
what future effects may do.

Directional gesture timing, physical strike adjudication, live Paredros session
integration, construction UI, authored material/process admission, dynamic graft
network reconciliation and gameplay save ownership remain consumer work.
Second-order operators, vows, Raise and generated world laws follow supported
cost and interruption semantics. An atomic batch is not a timed action: sustained
charging requires a product-owned action lifecycle and reevaluation on mutation.

## Progress

- 2026-09-28: documented rulings 404 to 407, including extensible fundamental
  effects, equivalent exchange, contextual scripts, ruleset exemplars and
  coordinated construction outputs. No code, cost formula, new runtime or
  adapter was implemented or selected.

- 2026-09-09: implementation lanes started; no acceptance receipt yet.

### First-slice receipt

- `shared/wing-functions`: 16 unit tests passed using its isolated
  `target-wing-functions` directory; zero doctests. Covers finite supply, stored
  charge reuse after source disconnection, atomic batches, graph admission,
  schema/identity validation, bounds, cycles, generation and mutation.
- `isometry-campaign --lib`: 42 tests passed on the combined tree, including
  construction proposal carriage through postcard generation records and
  present-body admission. Uses `.targets/wing-functions-campaign` to avoid
  unrelated host builds.
- `mesocosm-core --lib functions::`: two tests passed, covering subtree loss,
  replacement identities and body/network binary save/restore.
- `inspect_functions`: 50 seeds, 100 candidates (both forms), all 100 preserve
  serialization equality and lose techniques when an actuator is removed.
  Receipt: `../../testing/functional-generation/2026-09-09-batch.json`.
- `inspect_body_functions`: uses the existing seeded body generator and an
  explicit authored inspection profile. Removing part 3 also severs descendant
  part 4; both corresponding Strengthen requests become unavailable. Receipt:
  `../../testing/functional-generation/2026-09-09-body.json`.
- The body example initially caught a transient syntax error during another
  lane's Chronicle extraction. Its owner completed that edit; the example and
  campaign suite then passed on the combined tree. No foreign work was reverted.
- Combined publication review includes the shared-format extraction and live
  interchange tests. The earlier working-tree receipts remain scoped as stated.

### Design findings from implementation

The v1 evaluator uses deterministic first-fit breadth-first routing. This is an
explicit routing policy, not maximum-flow feasibility: some graphs can have
unused reachable supply requiring rerouting that v1 does not perform. Tests
preserve that boundary and the refusal explains selected-route capacity.

Batches are sequential and atomic. Edge capacities reset per operation, while
finite source/store charge is shared across the batch. Simultaneous multi-arm
swings need a timed action allocation policy before these values can represent
concurrent throughput. Store currently transfers charge; storing an executable
operator chain as an enchantment remains later work.

The two generated forms are functional blueprints over caller-admitted sites.
They do not yet construct staff geometry or allocate magical tissue. Keeping
that distinction explicit lets construction rules grow without making this
sampler another body authority or silently assigning magic from shape.
### Publication review, 2026-09-09

- Rechecked current live interchange: four tests passed after the independent
  typed-stock commit. No unrelated source changes were absorbed.
- Campaign construction now carries and validates an explicit network format
  version on decoding and inspection. Unknown versions are refused before use.
- The full ecology suite and native timed charging remain open as documented.
- Final campaign regression suite after the network-version fix: 43 passed.
