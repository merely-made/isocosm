// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Query {
    Alive(Binding),
    Trait {
        who: Binding,
        key: Key,
    },
    Account {
        who: Binding,
        key: Key,
        at_least: u64,
    },
    Below {
        who: Binding,
        key: Key,
        amount: u64,
    },
    Age {
        at_least: Tick,
    },
    Condition {
        key: Key,
        at_least: i64,
    },
    Part {
        who: Binding,
        revision: u64,
        part: Id,
    },
    Related {
        kind: Key,
    },
    /// The actor's mood, read from the world's needs (ruling 227), is at
    /// least `at_least`.
    Mood {
        at_least: i64,
    },
    /// The actor's mood is below `amount`.
    MoodBelow {
        amount: i64,
    },
    /// A body holds at least `at_least` matter, over every matter account it
    /// keeps (ruling 287).
    Holds {
        who: Binding,
        at_least: u64,
    },
    /// A body's wound share, out of a hundred, is at least `at_least`
    /// (`harm::wound_share`; the 2026-10-18 note to ruling 227).
    Wounded {
        who: Binding,
        at_least: u64,
    },
    /// At least `at_least` live members of the lineages `among`, the actor
    /// among them, reside at the actor's site as the pass began (S10). A
    /// traveller resides nowhere and counts none.
    Members {
        among: BTreeSet<Key>,
        at_least: u64,
    },
    /// Fewer than `amount` such members reside there.
    MembersBelow {
        among: BTreeSet<Key>,
        amount: u64,
    },
    /// The number of resident members of `among` per resident member of `per`.
    MembersPer {
        among: BTreeSet<Key>,
        per: BTreeSet<Key>,
        at_least: u64,
    },
    /// The actor has a live part expressing `function` (ruling 338). A
    /// process requires at most one, and binds the lowest-numbered such part
    /// as `Binding::Part`; its receipt reads that part's address.
    Expresses {
        function: Key,
    },
    /// An expression over what the act reads comes to something (X3): a
    /// mouthful of at least a milligram. It neither draws nor reads a kept
    /// value, being read before the act does either.
    Computed(Expr),
    /// A body's systems route `function` in `role` (rulings 569 and 575):
    /// one it carries names it there, a living part expresses it, and the
    /// systems naming it are realized as one.
    Routes {
        who: Binding,
        function: Key,
        role: Role,
    },
    /// Native reservation or incomplete selected work must be eligible
    /// before a material process can claim the actor's choice (837/838).
    Project {
        definition: Key,
        phase: crate::projects::Phase,
        process: Key,
    },
    /// A real polity decision is evaluated only inside native consent admission.
    ProjectDecision,
    InstitutionDecision,
    Institution {
        producer: Key,
    },
    /// The actor's sense reads at least `at_least` of `reading` (1071,
    /// Bodies 15), one of `senses::READINGS`, as the pass began. A world
    /// that does not sense reads nothing, so the requirement fails.
    Sensed {
        reading: Key,
        at_least: u64,
    },
    /// At least `at_least` members of `among` per `per_members` members of
    /// `per` reside at the actor's site (1145). With `per: None`, count the
    /// actor's current lineage, including after speciation. Both counts are
    /// living pass-start residents with cohort multiplicity. An empty
    /// denominator population never meets the requirement.
    MembersPerHead {
        among: BTreeSet<Key>,
        per: Option<BTreeSet<Key>>,
        at_least: u64,
        per_members: u64,
    },
    /// No matter remains in any body part (1114); an unbodied actor reads
    /// its entity ledger instead. Loose soil is not a rooted body's tissue.
    EmptyBody,
    /// Admitted resident food holders per member of the actor's current
    /// lineage, read through the named feed without appetite or clocks.
    FoodPerHead {
        feed: Key,
        at_least: u64,
        per_members: u64,
    },
    /// At least `at_least` units of edible matter per `per_members` members
    /// of the actor's current lineage at its site: the matter the named
    /// feed's admitted resident holders hold (`edible_matter`), not their
    /// number, set against living pass-start heads. An upper bound on what
    /// bites can take.
    EdiblePerHead {
        feed: Key,
        at_least: u64,
        per_members: u64,
    },
    /// A body has a living organ, its tissue held, whose port does `what` in
    /// a trade (`ecology::mutualism`, D39 model 5).
    Trades {
        who: Binding,
        what: crate::process::Trade,
    },
}
