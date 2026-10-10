# Wing organs: the hagioglyph and the impresa (2026-09-26)

**Status, 2026-09-28:** Carried 2026-09-26; carryover distinction 2026-09-28 (ruling 406). *(Brought current 2026-10-06 under ruling 618; the
earlier status line follows as written.)*

*Earlier:* **Status: carried, not rewritten.** This plan holds the general model plan's
§7.4, "Glyph canon, the journey, and divinity", and its impresa finding of
2026-09-21, moved here on 2026-09-26 under wing design record ruling 310
("Split out the organs") when the rest of the general model was archived at
[archive_docs/2026-09-26/](archive_docs/2026-09-26/2026-08-06_general_model_plan.md).
The words are the general model's; only the heading levels changed. Two
organs live here, both wing material no single game owns: the
**hagioglyph**, the divinity organ (the glyph canon, the journey, ascension,
the chosen referent and its periods, canon revisions; kernel
`shared/wing-glyphs`), and the **impresa**, the association organ (records
citing accepted events, read as fact, belief or legend; kernel
`shared/wing-impresa`). Gates G1 and G2 are implemented, G5 landed in Eponym
on 2026-09-14, G3 and G4 are open, G6 and G7 are ruled and not started, as
the status paragraph below records. The wing design record's §2.7 owns the
magic axes a world is drawn on and cites this plan for glyphs, journeys and
the impresa; the [sim plan](2026-09-22_sim_plan.md) §2.7 and §2.8 place both
organs among the sim's nouns; the
[glyph expression plan](2026-09-15_glyph_expression_plan.md) is the next
move under §1. The hagiograph, which the hagioglyph consumes, is the
[isoscape family plan](archive_docs/2026-10-10/2026-09-16_isoscape_family_plan.md)'s.

---

## 1. Glyph canon, the journey, and divinity (2026-09-13)

**Status: G1/G2 implemented and tested; G3/G4 open, 2026-09-13; G5 landed
in Paredros 2026-09-14; G6 and G7 ruled 2026-09-15, not started.** On
2026-09-15 Mark ruled that **hagioglyph** names this whole organ, the journey
to divinity, and that **impresa** names the association organ (G7). Mark
explicitly authorized a module usable by Mesocosm, Paredros, and Isometry.
Sharing this bounded machinery proceeds now; sharing the products' entire
rule evaluators is not required. The journey to divinity itself becomes the
new god's motif and constraint. This section owns progression and world laws;
the [presentation plan](2026-09-11_orthographic_voxel_presentation_plan.md#glyph-effects-and-interaction-experiment-2026-09-13)
owns the glyph experiments and their rendering evidence.

### World vocabulary and effect identity

A world admits a finite, versioned canon. Each base glyph has a stable ID,
display text, and a unique effect correspondence. The desired default covers
letters, numbers, and symbols; a world may use only `a`, `b`, and `c`, or a mod
may supply a much larger vocabulary. Arbitrary Unicode text is admitted as
display data. A character code, rendered shape, or font's glyph index is not
the gameplay ID. Explicit aliases and accent/variant definitions resolve to
a base and declared modifiers. Variants do not silently increase the set
needed for collection; a world wishing to require one makes it a base entry.
Font availability and legibility remain presentation validation, especially
for combining marks and visually similar signs.

The canonical correspondence is authored. A seeded shuffle permutes it while
preserving the one-to-one mapping, then the resulting definition is saved.
Reopening never reruns a newer generator over an inhabited world's meanings.
Effect IDs alone are not implemented effects: a product must admit a behavior,
cost, target rule, receiver interaction, and visible explanation for each.
The current three punctuation renderings are not an alphabet-sized spellbook.

Acquiring every base glyph in that world's canon qualifies the individual to
ascend or request a wish. An empty canon grants neither by collection. Other
routes, such as a named item or an extraordinary deed, are independent,
authored predicates; they need not invent missing glyphs. Whether a wish
consumes a collection, can repeat, or excludes later ascension remains a
world rule. A qualification receipt does not execute an unrestricted wish.
Canon changes during play require explicit migration of completion and
correspondence. They must not quietly revoke a god or make an old collection
complete under different meanings.

**G5, hagioglyph (Mark, 2026-09-14).** A glyph may vary over time by world
criteria: a canon revision the world publishes, caused by a period settling,
a hagiograph promotion admitted with its condition receipt, or an authored
epoch, and recorded in accepted history rather than rewriting it. Every
acquisition and grant keeps the revision it was accepted under; completion
and the ascension basis are judged against the journey's founding canon;
only the live effect of an owned glyph follows the current revision, so the
same glyph acquired at different times can carry different costs. The kernel
gains the revision stamp and a correspondence diff between two revisions;
Paredros is the first consumer, under its remembrance plan's stage F3b5.

**Hagioglyph names the organ (Mark, 2026-09-15).** Until this ruling the
word named only G5's time-varying glyph. It now names the whole of this
section's machinery: the canon, the journey, ascension, the chosen referent
and its periods, and the revisions above. This mirrors **hagiograph**, which
names the memorial organ rather than one memorial. A glyph under G5 is "a
glyph under revision" or "the live effect"; F3b5 in Paredros and R7 in the
effect pack preset plan are relabeled accordingly. The dependency stays
one-way: a hagiograph promotion is one cause of a canon revision, so the
divinity organ consumes the memorial organ and never the reverse. Crate
naming follows the ledger's tier rule; `shared/wing-glyphs` keeps its plain
name as the kernel under the organ.

**G6, chains and standing (Mark, 2026-09-15).** Effects compose into
chains, and the invoker's standing with a glyph is a per-individual,
per-glyph pair of ledgers, affinity and resistance, moved only by accepted
events under stated decay policies.

- A chain is a typed admission, the way the trophic grammar admits intake:
  links must be compatible by behaviour kind and receiver class, which the
  glyph experiment already requires before marks form strings, and chain
  length is a configured bound. Products adjudicate; the kernel owns the
  chain grammar and its receipts.
- A link's success reads affinity minus resistance and chain length, drawn
  from the seeded integer stream so replay holds. Success falls with chain
  length; a failure is an accepted event, never a silent roll.
- A failure raises the invoker's resistance to that glyph, capped, with a
  decay policy stated per world, so no run of ill luck locks a glyph for
  ever. Resistance is symmetric standing: it also applies against that
  glyph arriving from outside, so a failure becomes a defense.
- Affinity rises through the journey (provenance, successful invocations)
  and raises magnitude, including the self-targeted side effects an effect
  pack declares. High affinity is therefore dangerous when the side effects
  are unmanaged. Decay is evaluated lazily from the last event tick, never
  per tick, so it is deterministic and costs nothing at rest. Caps, decay
  and side-effect routing are explicit world policy, per the period rule
  above: no hidden bonuses.

**G7, impresa, the association organ (Mark, 2026-09-15).** "A god's
associations are their impresa, but other things could have theirs too."
Anything pointable, a glyph, a critter, a borg, a character, a faction, a
place, an item, a lot of nis, bears an impresa: the record of what it has
come to be associated with, and what has lapsed, over time. Plural imprese.
The expression table above is one static association, glyph to trait; the
impresa generalizes it across subjects and history.

- **One record type.** Subject identity, object identity, a kind, the canon
  revision, the cause receipt, the tick, and whether the record associates
  or lapses. Every record cites an accepted event; nothing is inferred from
  a name. Disassociation is a record, never a deletion.
- **Three readings of one record.** The accepted record is fact. An
  observer's held associations are belief, kept in the memory and belief
  layer Paredros already has, free to diverge from fact; reputation is
  belief. The hagiograph's promotions of associations that met a condition
  are legend. No reading is a second authority.
- **Kinds are a world-configured closed set**, seeded from claim, discover,
  experience, embody, invoke, defeat, and extended by packs the way the
  canon is; directional, subject to object. A lapse by decay is a record
  emitted at settlement, like a period.
- **Consequential records never compact.** A record cited by a later
  accepted event, by a god's ascension basis or by a faction's stance is
  retained; the rest fall under the retention budgets and compaction the
  owners paragraph already anticipates.
- **Cross-vessel, imprese travel as records**, which satisfies
  choices-not-morphology and pointable inheritance; each vessel maps kinds
  to its own meaning.
- **Home.** A plain `wing-` crate beside `wing-glyphs` owns the record type
  and its validation only; products own storage in their accepted history
  and derive per-subject and per-glyph readings as projections. Extraction
  into mere's linked-data edges, whose evidence-sense vocabulary landed on
  2026-09-15, waits for two consumers, and the kinds are designed so that
  projection stays possible. Prior art to read before the kinds are ruled:
  Dwarf Fortress legends mode and Caves of Qud's sultan histories. The
  Nemesis boundary stands: remembered grudges are fine, rival promotion
  hierarchies are not.

**Done when (G6):** a two-link chain on compatible declarations admits and
an incompatible one refuses by name; success, failure and the resulting
resistance change replay identically from a restored save; resistance
decays to its floor on a stated policy and blocks an incoming effect of the
same glyph in the meantime; a high-affinity invocation reports its declared
side effect on the invoker. **Done when (G7):** two vessels append imprese
for one identity through the shared record type without sharing a world;
a lapse and a re-association on the same pair are both pointable with their
causes; the belief reading of a faction diverges from fact after a witnessed
event and reconciles on a recorded one; a consequential record survives a
compaction that removes an unreferenced one.

**G5 amended, hagioglyph is the organ (Mark, 2026-09-15).** The paragraph
above names one mechanism, a glyph whose live meaning follows a published
canon revision. That mechanism is now called **canon revision**, after the
kernel's `canon_revision` stamp and `Canon::correspondence_diff`, and no
longer owns the word. **Hagioglyph** is the wing's glyph organ as a whole:
the canon and its revisions, expression through traits, the journey,
experience, standing (affinity and resistance, to be ruled) and ascension,
with `wing-glyphs` as its kernel and each product's glyph reading as a
consumer. The dependency arrow is unchanged: hagiograph promotes, a canon
revision follows, the hagioglyph reads it. Paredros's stage F3b5, its
document-host glossary line and the `eponym-world` glyph reading's
module comment carry the narrow sense and relabel to canon revision on
the Paredros side.

### An individual has a journey, not just an inventory

**2026-09-28 refinement (406):** the journey's continuity is distinct from
the conditions selected for the next life. An old incident may persist as a
detriment, an absent part, or a related benefit or characteristic ability.
Selection may be randomized or configured by the player, preferably per
condition; a blanket keep-conditions option is also acceptable. Leaving a
condition behind does not erase its event or acquisition evidence. The
[body contract](2026-07-31_wing_phenotype_contract_plan.md#conditions-carried-into-a-new-life)
owns embodiment adaptation and the proposed selection record. This does not
change the existing eligibility for reincarnation or make the progression
kernel an implementation of condition transfer. Its restrictions, defaults
and interaction with acquisition restoration remain to be designed.

Record original acquisition order, incarnation, simulation tick, and accepted
source evidence, including the means: ability, trait, technique, item, bond,
quest, event, or a mod-defined category. Losing an item is distinct from
forgetting that one learned through it. Reacquisition never rewrites the
first acquisition. Critter, borg, and character are all eligible; the record
belongs to the continuing individual, independent of taxonomy and controller.
Accepted append order is authoritative; source tick metadata never reorders
the journey across different clocks. `ascension_basis()` retains the exact
grant prefix at first ascension, even when later lives add further evidence.

Ascension enables reincarnation of that individual. It preserves the journey
while starting a new body's experience progression. Configured experience
thresholds restore inherent access in original acquisition order. Provenance
remains available at each unlock so a bond-earned effect can return as a
bond-dependent ability, while a practiced technique can require performance.
The later bag may expand without rearranging earlier acquisitions.

The same collection can therefore produce different divinities. As an
illustrative rule, a rescuer who learned through sheltering others might
recover protection through maintained bonds; a survivor who acquired that
glyph through enduring fire might recover it through exposure and endurance.
These are authored possibilities, not judgments inferred from event names.
Any generated affinity, cost, obligation, exception, or restriction must cite
the acquisition evidence and rule revision supporting it. Grouping acquisition
categories is an inspectable motif summary; it is not yet that rule evaluator.
Show the likely constraints before commitment, and retain the player's means
of understanding, fulfilling, or deliberately challenging them.

### Chosen divine power and periods

Collection determines available vocabulary. Divine power depends on the
chosen referent: a condition, event, entity kind, or another admitted metric.
The god chooses its source and measurement period before the period begins,
then chooses again at its end. The definition includes world/reference scope,
predicate revision, unit, aggregation, and the simulation interval `[start,end)`.
A name such as "popular" is insufficient: event occurrences, distinct actors,
and entity-ticks are different quantities. Products supply accepted observations;
the shared ledger never samples a renderer, wall clock, or arbitrary biography.

For the first kernel, the explicit policy is accumulation followed by settlement
at the end. Every simulation tick contributes one aggregate, including zero.
Missing ticks cannot be mistaken for zero; duplicate or out-of-order input is
rejected. Choice stays fixed, arithmetic is checked, and settlement is one-time.
A long period delays access and accumulates more of a continuing source. A
short period offers earlier access and an earlier chance to change dependence.
With identical observations, dividing a period does not multiply its total.
Additional interest, decay, averaging, caps, stockpiles, spending, and ongoing
access during a period need explicit world policies rather than hidden bonuses.

Journey and referent are distinct but connected. A world's journey rules may
restrict eligible referents, alter their costs, or impose obligations; a chosen
referent does not erase the history that made the god. Reference definitions
and active commitments must survive death, replay, and save restoration under
the product's chosen reincarnation rules. The first accumulator proves period
mechanics, not persistent divine economy or a universal predicate language.

### Owners and implementation gates

`shared/wing-glyphs` owns canon validation, deterministic correspondence,
acquisition history, ordered unlocks, explicit progression transitions, and
fixed-period accumulation. It has no graphics or product dependencies.
Cloned canons share immutable indexed storage, with logarithmic identity
lookups. Individual journeys store their acquisitions and references without
copying the entire vocabulary. Exported snapshots embed the full canon for
standalone restoration; a future world save can deduplicate that carriage.
Canon size and journey retention budgets are explicit configuration, not a
fixed alphabet ceiling. Large mod canons can raise those budgets and the JSON
ingress limit. Every successful JSON export must fit its corresponding import
budget. Rejected grants leave evidence intact and report exhaustion; silently
evicting an original acquisition would change the future god. Longer-lived
worlds will need compaction preserving those consequential records.
`wing-functions` keeps body-bound resource routing and operator receipts;
`wing-formats` keeps primitive interchange framing. Neither needs to become
a magic scheduler. Glyph display and strings consume admitted effect facts;
their color, animation, and generated layout cannot award an acquisition.

Mesocosm's optional disposable trial maps positive accepted movement, feeding,
and carving records through configured grant rules. Actual history ordinal,
tick, and organism identify the evidence. Trial reset reconstructs the same
journey. This is an integration experiment, not a new field in durable `World`
or a playable reincarnation feature. Isometry's system-resolution boundary
owns adjudication; Paredros's continuing subject identity is distinct from a
body revision and from succession into another existing life.

1. **G1, shared kernel, implemented:** validate small/empty/Unicode canons, variants and
   seeded bijections; preserve first acquisition, source, and order; prove
   non-vacuous completion, reincarnation unlocks, fixed-period totals, rejection
   without mutation, and validated journey restoration. The period accumulator
   has inspection serialization only; durable period restoration is G4.
   No effect execution claim.
2. **G2, real consumer, implemented:** optional Mesocosm rules consume accepted world events;
   refused actions cannot grant the action's glyph; baseline/reset/replay and
   repeated reads reproduce the same records. No player-succession shortcut.
3. **G3, usable vocabulary:** admit a default effect pack and two contrasting
   journey rules through product adjudication. Explain their costs and motifs,
   execute receiver interactions, and show distinguishable spatial results.
4. **G4, durable lives and divinity:** products admit world/canon/journey save
   ownership, alternate ascension and wish policies, same-individual rebirth,
   active source commitments, and spending. A second product exercises the
   shared API with its own identity and law rather than importing Mesocosm's
   ecology. Mods and world migrations have explicit rejection and repair paths.

### First implementation receipt

The new MPL-2.0 `shared/wing-glyphs` crate passed **16 tests** at ac38887 (28 as of 2026-09-15, after the pack and expression tables), including a
literal seeded-shuffle vector, opaque Unicode and explicit variants, a
5,000-entry canon with shared storage, first-acquisition preservation,
divine-only rebirth, immutable ascension evidence, snapshot transition replay,
and five fixed-period tests. The period tests include zero support, ordering,
overflow, one-time settlement and additive short/long periods. Snapshot replay
checks consistency, not the authenticity of a caller's evidence.

Mesocosm's final release runtime suite passes **44 tests**, including five new
consumer tests. They exercise real accepted carving, movement and feeding,
zero-removal/rejected actions, unchanged core history/hash, exact reset/replay,
and actual control transfer followed by another body's carve. The collection
does not follow that control transfer.

The [public-API example](../crates/mesocosm-runtime/examples/glyph_journey.rs)
produces [this JSON receipt](../testing/glyphs/journey.json). An unmodified
seed-0 founder removes 15 voxels at `[0,12,0]`, earning the one configured base
glyph. Its four-tick metric observations are `15,0,0,0`; settlement is 15.
The copied shared journey qualifies for both routes, ascends, reincarnates,
retains its original source, and restores inherent access at 10 experience,
with none at 9. Experience is explicitly supplied to this isolated probe.
The core world does not reincarnate, award that experience, execute the
referenced effect, or mint divine power. Baseline hash is `765c055b377dfb62`;
the ordinary four-action world ends at `ea76e63d1fb9fa13`.

Validation commands were `cargo test --offline` from `shared/wing-glyphs`, and
`cargo test --release --frozen -p mesocosm-runtime --lib -j3` plus
`cargo run --release --frozen -p mesocosm-runtime --example glyph_journey -j3`
from `mesocosm`. The product lockfile is locally generated under its existing
ignore policy; the standalone shared crate's lockfile is tracked. These are
CPU correctness receipts. This slice changes no native view and supplies no
new visual or performance acceptance claim.


## 2. Findings

- **2026-09-21, from the wing design record's rulings 80 and 82:** the sim's
  middle tier of keeping, "of note", is ruled to be a literal note on a
  thing, and the note is ruled to be the impresa record, so a thing is of
  note while it bears a live impresa and collection is lapsing. Mark also
  wants a freeform text field beside the generated details, format a
  setting, djot preferred, edited what-you-see; knot-editor and
  `knot-document` are the stack's djot authority. `wing-impresa`'s `Record`
  is closed today (`deny_unknown_fields`, schema version 1) and holds to
  "transcription and never inference", which authored text is not. Owed
  here: whether the text is an optional field on the record or a sibling
  record beside it, and the version bump either way. **Updated the same
  day, ruling 85:** Mark proposes "closed subset, open superset". The
  closed `Record` stays as it is and a note is an open envelope around it in
  the storing product, so nothing is owed in this crate beyond, at most, a
  world adding a kind for a player's own jotting.


## 3. Progress

- **2026-09-26:** carried here from the general model plan's §7.4 and Findings
  under ruling 310; the general model archived the same day. No design changed.
