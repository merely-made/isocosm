# Functional generation: magic as world operations

**Status:** 2026-10-30: F10 to F12, the VTT's edition procedures, moved to `vtt-overlay`. Cut from `2026-09-09_functional_generation_plan.md` and its effects-remainders item from `2026-09-11_orthographic_voxel_presentation_plan.md`, which are archived in
`archive_docs/2026-10-28/`. The first slice is built: `shared/wing-functions` (network, operators, generation, carriage) is shared and used by `shared/cosmograph` (`effects/functions.rs`, `systems/carry.rs`) and by Eponym. The composition design (rulings 404 to 407, case 1009) is written; no cost formula, script runtime or ruleset adapter is selected. F1 to F9 are not a priority (parked; `wing-organs` D1).

**For:** magic and construction as world operations: the six axes a generated magic system is drawn on (case 1009, the nucleus), fundamental effects with equivalent-exchange cost, effect scripts connected to the world, persistent-effect lifecycle, rulesets and ways of casting, nested construction composition, the inspection a player or developer needs, the first slice's open work, and the effects system's remainders.

Not here: glyph canon, journey, ascension, chains and standing, the impresa (`wing-organs`); the body contract and conditions (`wing-phenotype-contract`); capability, repertoire and proficiency (`sim`); any speed or tick-cost bar, including the cost of bounding script work (`sim-execution`); licences of rulesets that may ship (`content-packs`); the VTT's effect catalogue and edition procedures, F10 to F12 (`vtt-overlay`).

## Constraints

- **Parked**, with magic: not a priority now, and open to design and code, which are what unpark it (`wing-organs` D1).
- **Equivalent exchange (ruling 404, Mark):** "Equivalent exchange. Any effect (burn, freeze, shock, buff, debuff, heal…) should be available as a cantrip and combinable. But each level of combining takes a magnitude more energy." Cantrip names a small expression, not universal access, zero cost or a ruleset's casting category. Equivalent exchange is a world rule, not a claim of thermodynamic realism.
- **Construction declares function; geometry supplies measurements (405).** Reach, clearance, contact and coverage come from geometry, so a body change can invalidate a reading. Constructible, executable in these circumstances, and advantageous here are different questions. The current single-part bindings and three-operator evaluator are foundations, not the complete system.
- **Reads never silently regenerate an inhabited world's rules.** Reads use accepted state and admitted definitions; mod-authored and generated definitions share one admission and inspection path. The sim's rule stands: lower to process definitions where possible; hooks without a supported lowering stay foreground-only.
- **Ordinary magic suspends no invariant** of the world (case 1009, cost and breach axis); any breach is priced and leaves residue.
- **Fairness means understandable exchange**, not equal builds and not removing every interaction players exploit. Self-amplifying interactions are not rejected for being strong; their accounting, feedback and cost must be inspectable.
- **Developer explanations stay distinct from what an inhabitant knows.** An execution trace alone does not establish an actor's motives or beliefs. Cosmograph's staged, checked effects remain the authority boundary.
- **Charge units are authored and abstract** and do not replace Mesocosm's conserved matter ledger. `wing-functions` owns neither a second body nor product world state. Game adjudication stays with each product.
- **No new ECS, skeletal library or physics-learning system follows from the composition model.**
- **Ars Magica's open text informs; nothing is copied into the sim** (ruling 522, doctrine Licensing 1). What ships from it is share-alike and is `content-packs`' to clear.

## Phases

### F1. The six axes of a magic system
A generated magic system is one trait of the world's genotype, drawn at founding or grown at a boundary, on six axes (case 1009, D30, accepted 2026-09-22): *source* (the world's fields, the caster's anatomy, a bearer, a god, the record's frequency); *gate* (unconditional, anatomical, acquired on a journey, skill-gated, social, placed); *orientation* (bending how a process runs, or yielding an effect outright); *cost and breach* (which account pays, which of the world's invariants the system may suspend, at what price, leaving what residue); *rhythm* (cyclical, continuous, once); *manifestation* (how it shows and is sensed). Divinity reads such a system through its effects, acquisition and frequency, so the ladder of sacrificed forms follows the gate axis. Mod authors and the generator should both be able to create such magic. Placed here at Mark's word, 2026-10-22: "Axes to the plan".
- Done when: not stated; owed
- Needs: F2, F3, F4
- Rulings: 1009, 101, 102, 49
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:118-132`; design record case 1009 (`design_docs/2026-09-18_wing_design_plan.md:10933`)

### F2. Cost curve for combining
Strongly escalating cost with deeper or stronger combinations; the exact curve is open. No tenfold growth, no numerical definition of composition level and no D&D or Pathfinder spell rank is selected; ten sequential low-power casts are not one tenth-level combination. Reading, not ruled: costs should expose their contributing effects, strengths, composition relationships and contextual modifiers; composition depth, repeated effects, duration, area, range, simultaneous versus sequential use and persistent propagation need explicit treatment before a formula is chosen. Explainable, surprising and powerful combinations are wanted.
- Done when: not stated; owed
- Needs: F3
- Rulings: 404, 279
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:45-70`

### F3. Effect scripts connected to the world
A script has a fundamental character plus characteristics that vary with the sim (nearby fire makes fire hotter and cheaper; ice spreads until sunlight stops it; world size influences gravity). These are admitted alternatives a world may generate, not defaults. Mark: "basically hook other generators of the sim into the effect's script calculation." Proposed execution contract, not a selected runtime: an effect definition names its revision, parameters, world inputs, targets, operations, costs, ongoing behaviour and explanation, and the facts it reads and may change. Open: "How to bound script work without silently changing world outcomes remains an execution-design question, not a balancing rule."
- Done when: not stated; owed
- Needs: `sim` process definitions (lowering); `sim-execution` for any cost bar on script work
- Rulings: 404
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:72-98`

### F4. Persistent-effect lifecycle
"an explicit lifecycle: activation, due updates, interruption and termination, including the event or condition that can stop them. Reevaluate a dependency when it changes, and schedule ongoing work at the mechanic's required grain. Self-amplifying interactions are not rejected merely for being strong; their resource accounting, feedback and execution cost must be inspectable." An atomic batch is not a timed action: sustained charging needs a product-owned action lifecycle and reevaluation on mutation.
- Done when: not stated; owed
- Needs: F3; a product-owned action lifecycle (Eponym's `timed_action.rs` is one, see Findings)
- Rulings: 404
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:92-98`, `:192-203`

### F5. Rulesets and ways of casting
Schools and aspects are interpretations of effects and their relationships. Mark's references (Homestuck, Elder Scrolls, D&D, Pathfinder, PbtA) are influences to compare, not interchangeable schemas. A ruleset's supplied spells are exemplars whose defining behaviour is preserved in the mapping; generated spells fill out the catalogue; no mechanical equivalence is inferred from a school label. CoreRPG and the individual system adapters are unresolved. The same canonical effect need not acquire the same spell level, damage or casting procedure under every ruleset, but the overlay calibration requirements still apply to shared world outcomes. Casting methods are open; no universal mana pool, gesture, preparation or progression model is selected. Ball x Pit is Mark's analogy for combinations, not an algorithm. Ruling 522 (2026-10-03) gave Ars Magica's Hermetic magic (Techniques composed with Forms at guideline levels) as a reference. Whether its structure becomes native vocabulary is a question below.
- Done when: not stated; owed
- Needs: ruleset adapters (`vtt-overlay` rulesets, `mesocosm-overlay` calibration); F1
- Rulings: 404, 522
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:100-116`; design record ruling 522 (`design_docs/2026-09-18_wing_design_plan.md:3722`)

### F6. Construction composition
Nested assemblies with addressable internals, shared by critters, plants, props, equipment and buildings. Compatibility describes relationships and conditions, not a list of permitted finished designs. The same accepted construction and revision support functional readings, generated form, colliders, animation bindings and descriptions; structural attachment, resource transport and control relationships need not be one graph. Motion can use reusable stylized procedures that respect the construction. Mark (407): "capability, repertoire, proficiency. agreed"; perception and self-perception sit with belief and tenets, including believing oneself unable to do something actually possible.
- Done when: not stated; owed
- Needs: `wing-phenotype-contract` (identity and contextual embodiment); `sim` (capability, repertoire, proficiency); `bodies`
- Rulings: 405, 407, 462, 492
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:22-43`

### F7. Design completion and inspection
Before any implementation slice is selected, complete the description of effect composition, context reads, resource exchange, ongoing actions and revision changes across the construction, sim and overlay contracts, and record which parts are authored, generated, derived once, invalidated by change, scheduled or continuously resolved. An examiner traces an action's support to parts, equipment, procedures and conditions, shows cost contributions and refusal reasons, and connects accepted outcomes to causes. Moirai (scheduled transitions, property-gated reactions, execution ancestry) was reviewed as comparison only: no benchmark, no dependency choice.
- Done when: not stated; owed
- Needs: F2, F3, F4, F6
- Rulings: 404, 407
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:134-151`

### F8. Open work after the first slice
Consumer work: directional gesture timing, physical strike adjudication, live Eponym session integration, construction UI, authored material and process admission, dynamic graft network reconciliation, gameplay save ownership. Behind cost and interruption semantics (F2, F4): second-order operators, vows, Raise, generated world laws. Storing an executable operator chain as an enchantment (Store only transfers charge today). A timed-action allocation policy for simultaneous multi-arm swings. The v1 evaluator routes first-fit breadth-first and is not max-flow, so some graphs have reachable supply it never reroutes. Generated forms are blueprints over caller-admitted sites and do not construct geometry or allocate magical tissue. The full ecology suite and native timed charging were left open. See Findings: part of this list may already be closed in Eponym.
- Done when: not stated; owed
- Needs: F2, F4
- Rulings: 404 to 407
- From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:168-203`, `:239-263`

### F9. Effects system remainders
The effects receiver is a demonstration plane with scripted contact age. Open: surface and item identities, organ emission sites, scene interaction, CSS effect styling, sound, mixed fixed and generated rule packs, effect-to-effect composition, world collision and material or item responses, gameplay rule packs and guarantees, transparent haze and overlap policy, surface editing and picking, soundscape event consumption, richer glyph assets, population cost measurement. Each needs owner-specific acceptance, and game adjudication stays with each product. A later local audio consumer owns sound mappings, aggregation, audio-clock scheduling and voice budgets; Hocket (under woodshed/ports/hocket) is prior art, and its looper engine is not the event model.
- Done when: not stated; owed (each item needs owner-specific acceptance)
- Needs: `wing-organs` G3 (the default effect pack) and G8 (the catalogue); `orthographic-voxel-presentation` for the renderer
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-11_orthographic_voxel_presentation_plan.md:1915-1920`, `:2207-2214`, `:2305-2310`

## Decisions

- **D1, effect catalogue (2026-10-28), applies Processes 3 and VTT 5:** Mark: "Ok! Agreed!" In response to the proposal: "I'd build the catalogue broadly, then make a small set work completely across the three rulesets. Body transformation, senses and persistent enchantment would expose useful differences immediately while giving world generation something substantial to grow around."

  Outcome: applies to doctrine Processes 3 and VTT 5. F10a and F10b delivered catalogue and native controls only; Mark's follow-up requires the agreed executable examples, implemented in F11. The VTT lifecycle addresses F10c's temporary lasting effects; complete edition ports, native magic and permanent device creation stay explicit open work. Speculative metamorphoses remain optional candidates, not selected world laws.

  Links: area Processes and rules · draws on 539, 540, 541, 543 · plans functional-generation · summary Start a broad effect catalogue with explicit edition correspondences and a bounded executable tranche.

## Deferred and parked

- **F1 to F9** remain parked with magic (`wing-organs` D1). F10 to F12, the catalogue and edition procedures D1 approved, are `vtt-overlay`'s since 2026-10-30. The first slice's lanes (evaluator, generation, consumer wiring) need no further work except as F8 records.

## Questions for Mark

- **Exact cost curve (404)**: "the exact curve is open." The old plan rules out tenfold growth and any D&D or Pathfinder rank as selections and names the quantities to treat explicitly (F2). It offered no candidate curve. From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:45-70`.
- **Casting methods and ruleset adapters**: no universal mana pool, gesture, preparation or progression model is selected; CoreRPG and the individual system adapters are unresolved. From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:100-116`.
- **Bounding script work**: how to bound script work without silently changing world outcomes. From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:72-98`.
- **Lasting effects as addressable world objects (raised 2026-10-27)**: today a persistent effect is only a lifecycle inside a script and a chain's links are receipts; nothing makes a running enchantment, ward or curse a pointable thing that other effects can find, name, suspend or cancel. Is a lasting effect an entity or record in the world (addressable like an impresa subject, with provenance and a cause receipt), so detect, dispel and counter are ordinary operations against it, and what ends it besides its own termination condition? Attaches to F4 and F7; possible homes of addressability are `wing-organs` G7 and, as the counter's existing seed, G6 resistance. From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:92-98`, `:134-143`.
- **A perception layer apart from truth (raised 2026-10-27)**: perception is per creature (rules 5 and 59) and "believing oneself unable" sits with belief and tenets (407), but magic has no axis for altering what a perceiver receives without altering the world. Does an illusion write to a perceiver's sensed or believed layer (a note on the knower) while the world fact stays true? Is that the manifestation axis of F1 or a separate layer? How does it meet the impresa belief reading (`wing-organs` G7) and Eponym survival mode, which "shows only what the creature knows" (`eponym-overlay` E3)? From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:118-132`; doctrine Organs 3.
- **Ars Magica 5e as the native vocabulary (raised 2026-10-27)**: adopt its structural vocabulary (Technique x Form composition, level guidelines, casting methods formulaic, spontaneous and ritual, Arts and Realms) as the native grammar of the sim's magic, with the Hermetic tradition expressed as one preset over the six axes and not the baseline? This reverses or refines 522's "nothing copied", so it needs a ruling on what is structure (adoptable) versus content. It also sets the additive level arithmetic against 404's "a magnitude more energy" per level of combining (F2). From: `archive_docs/2026-10-28/2026-09-09_functional_generation_plan.md:100-132`; design record ruling 522.
- **Structurally different worlds**: the founding charter's question (`eponym-overlay` charter questions) whether worlds differ structurally touches world-dependent scripts (F3) and the axes (F1); recorded there, not restated here.

## Findings

- 2026-10-28: `shared/wing-functions/src` holds carriage, generation, network and operators with tests. It is consumed by `shared/cosmograph/src/effects/functions.rs`, `shared/cosmograph/src/systems/carry.rs`, `eponym/crates/eponym-play` and `eponym-client`, and by `crates/isocosm-campaign`. The "first slice implemented" header and "no implementation lane open" describe two different things: the evaluator exists; the composition design is unbuilt.
- 2026-10-28: Eponym owns deterministic limb actions over the shared router (`eponym/crates/eponym-play/src/timed_action.rs`: elapsed ticks, anatomical bindings, interruption, strike receipts). So F8's "directional gesture timing", "physical strike adjudication", "live Eponym session integration" and "native timed charging" are at least partly built there; which F8 items they close is not verified. The strike-quality policy choice is `eponym-overlay`'s (ep.12 there).
- 2026-10-28: the first slice's consumer wiring named `mesocosm-core`'s body membership adapter, which no longer exists; the native equivalent is `shared/cosmograph/src/effects/functions.rs`. First-slice test counts and the batch receipts are not gates.
- 2026-10-28: the registry carries fifteen functions in `rules::default_functions`, five bound to native processes; effects key on `function:<name>` (773). That is the existing hook between F3 scripts and the sim.

## Progress

- 2026-10-30: F10 to F12 moved to `vtt-overlay`.
- 2026-10-28: cut from the functional generation plan and the presentation plan's effects remainders (pres.32); all phases parked.
