// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The five natives as definitions (ruling 262): Mesocosm's physiology
//! lowered into processes over a minimal body (rulings 446, 453 and 455),
//! its numbers Mesocosm's own, written into the expressions as a world's
//! rules carry them. Contract prices rent and the mouthful by its span;
//! intake is the meal, routed by TD5 and paid for where the prey's gland is
//! charged; fix draws its income from the site's soil; secrete is the gland
//! the lowered script develops and the rent and dose it adds. Contract's
//! reach, sense's perception, and fix's forage radius and crowding wait for
//! places, so sense runs nothing yet.

use crate::{generate::process, rules::*, schema::*};
use std::collections::BTreeSet;

/// Mesocosm's reference body: 100 mg in a segment of 125 voxels.
const REFERENCE_MASS_MG: i64 = 100;
const REFERENCE_SEGMENT_VOXELS: i64 = 125;
/// The reference mass to the three-quarter power, as Mesocosm takes it.
const REFERENCE_MASS_34: i64 = 31;
const UPKEEP_BASE_MG: i64 = 1;
const UPKEEP_SCALE: i64 = 62;
const FIXES_BASE_MG: i64 = 5;
const GRAZES_BASE_MG: i64 = 3;
/// A body this light or lighter starves.
pub(super) const STARVATION_MG: u64 = 20;
/// TD5's horizon: a meal burns into the reserve while it holds fewer
/// ticks of upkeep than this.
const STARVED_UPKEEP_TICKS: i64 = 100;

pub(super) const SOIL: &str = "world:soil";
pub(super) const CANDIDATE: &str = "ability:candidate-secrete";
const CONTRACT: &str = "function:contract";
const INTAKE: &str = "function:intake";
const FIX: &str = "function:fix";
const SECRETE: &str = "function:secrete";
/// What a meal took and what its prey held before it, kept for the dose.
const TAKEN: &str = "probe:taken";
const PREY: &str = "probe:prey";

pub(super) fn tissue(lineage: u32) -> Key {
    format!("tissue:{lineage}")
}

pub(super) fn reserve(lineage: u32) -> Key {
    format!("reserve:{lineage}")
}

fn c(v: i64) -> Expr {
    Expr::Const(v)
}

fn add(v: Vec<Expr>) -> Expr {
    Expr::Add(v)
}

fn mul(v: Vec<Expr>) -> Expr {
    Expr::Mul(v)
}

fn div(a: Expr, b: Expr) -> Expr {
    Expr::Div(Box::new(a), Box::new(b))
}

fn least(v: Vec<Expr>) -> Expr {
    Expr::Min(v)
}

fn most(v: Vec<Expr>) -> Expr {
    Expr::Max(v)
}

fn less(a: Expr, b: Expr) -> Expr {
    add(vec![a, mul(vec![c(-1), b])])
}

fn at_least(a: Expr, b: Expr) -> Expr {
    Expr::AtLeast(Box::new(a), Box::new(b))
}

fn held(who: Binding, key: &str) -> Expr {
    Expr::Read(Reading::Account {
        who,
        key: key.into(),
    })
}

fn computed(e: Expr) -> Amount {
    Amount::Computed(e)
}

/// A body's adult mass (TD6, ruling 455): each living part's voxels priced
/// at the reference mass a segment, floored and at least a milligram, and
/// at least a milligram in all, as Mesocosm's build multiple reads it.
fn ceiling() -> Expr {
    let voxels = Expr::Read(Reading::Voxels { who: Binding::Part });
    let priced = div(
        mul(vec![voxels, c(REFERENCE_MASS_MG)]),
        c(REFERENCE_SEGMENT_VOXELS),
    );
    let each = Box::new(most(vec![c(1), priced]));
    most(vec![
        c(1),
        Expr::Parts {
            who: Binding::Actor,
            each,
        },
    ])
}

/// Mass to the three-quarter power, Mesocosm's integer way.
fn three_quarter(mass: Expr) -> Expr {
    let m = || most(vec![c(1), mass.clone()]);
    Expr::Sqrt(Box::new(mul(vec![m(), Expr::Sqrt(Box::new(m()))])))
}

fn span() -> Expr {
    Expr::Read(Reading::Span {
        who: Binding::Actor,
        function: CONTRACT.into(),
    })
}

/// TD7's rent and PD2's gland: one milligram and the mass's share, priced
/// by the actuators' swing and the gland's held mass per ceiling.
fn upkeep(i: u32) -> Expr {
    let m34 = three_quarter(held(Binding::Actor, &tissue(i)));
    let gland = Expr::Read(Reading::CellMass {
        who: Binding::Actor,
        function: SECRETE.into(),
    });
    let priced = add(vec![
        ceiling(),
        mul(vec![span(), c(REFERENCE_MASS_MG)]),
        gland,
    ]);
    let share = div(
        mul(vec![m34, priced]),
        mul(vec![c(UPKEEP_SCALE), ceiling()]),
    );
    add(vec![c(UPKEEP_BASE_MG), share])
}

/// TD5's one rule: whether the reserve holds fewer ticks of upkeep than
/// its horizon.
fn hungry(i: u32) -> Expr {
    let budget = mul(vec![upkeep(i), c(STARVED_UPKEEP_TICKS)]);
    at_least(budget, add(vec![held(Binding::Actor, &reserve(i)), c(1)]))
}

/// How far an account is below the ceiling.
fn gap(key: &str) -> Expr {
    most(vec![c(0), less(ceiling(), held(Binding::Actor, key))])
}

/// What a body has room for (TD6): below its ceiling in tissue and in
/// reserve.
fn room(i: u32) -> Expr {
    add(vec![gap(&tissue(i)), gap(&reserve(i))])
}

/// A producer's income (TD2c): an allometric rate, at least a milligram,
/// within its room. Crowding waits for places.
fn income(i: u32) -> Expr {
    let m34 = three_quarter(held(Binding::Actor, &tissue(i)));
    let rate = div(mul(vec![c(FIXES_BASE_MG), m34]), c(REFERENCE_MASS_34));
    least(vec![most(vec![c(1), rate]), room(i)])
}

/// TD9's mouthful: the grazing rate scaled by the build multiple, within
/// the room.
fn mouthful(i: u32) -> Expr {
    let m34 = three_quarter(held(Binding::Actor, &tissue(i)));
    let priced = add(vec![ceiling(), mul(vec![span(), c(REFERENCE_MASS_MG)])]);
    let rate = div(
        mul(vec![c(GRAZES_BASE_MG), m34, priced]),
        mul(vec![c(REFERENCE_MASS_34), ceiling()]),
    );
    least(vec![most(vec![c(1), rate]), room(i)])
}

/// TD5's landing of what the act holds in hand of `hand`: into the reserve
/// first when hungry and the tissue first otherwise, each up to its
/// ceiling, then what will not fit back to the site (TD6).
fn landing(i: u32, hand: &str, conversion: Conversion) -> Vec<Effect> {
    let into = |to: Key| Effect::Convert {
        who: Binding::Actor,
        from: vec![hand.into()],
        amount: computed(least(vec![held(Binding::Actor, hand), gap(&to)])),
        to,
        conversion,
    };
    vec![
        Effect::When {
            guard: hungry(i),
            then: vec![into(reserve(i)), into(tissue(i))],
            otherwise: vec![into(tissue(i)), into(reserve(i))],
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: hand.into(),
            amount: computed(held(Binding::Actor, hand)),
        },
    ]
}

fn own(i: u32) -> Query {
    Query::Trait {
        who: Binding::Actor,
        key: format!("ability:probe-{i}"),
    }
}

fn expresses(function: &str) -> Query {
    Query::Expresses {
        function: function.into(),
    }
}

fn due(id: &str, causation: Causation, effects: Vec<Effect>, priority: i32) -> Process {
    let mut p = process(id, causation, effects);
    p.period = Some(1);
    p.priority = priority;
    p
}

/// Rent from the reserve before the tissue, returned to the site as soil
/// at once, as Mesocosm returns it (ruling 446).
fn upkept(i: u32) -> Process {
    let rent = Effect::Spend {
        from: vec![reserve(i), tissue(i)],
        to: Binding::Place,
        amount: computed(upkeep(i)),
        into: Some(SOIL.into()),
    };
    let mut p = due(
        &format!("body:upkeep-{i}"),
        Causation::Choice,
        vec![rent],
        0,
    );
    p.requires.push(own(i));
    p
}

/// A producer fixes: its income drawn from the site's soil, shared out when
/// the soil runs short (ruling 454), synthesized and landed by TD5.
pub(super) fn fix(i: u32) -> Process {
    let draw = Effect::Transfer {
        from: Binding::Place,
        to: Binding::Actor,
        account: SOIL.into(),
        amount: computed(income(i)),
    };
    let effects = [vec![draw], landing(i, SOIL, Conversion::Synthesis)].concat();
    let mut p = due(&format!("body:fix-{i}"), Causation::Choice, effects, 1);
    p.requires.extend([own(i), expresses(FIX)]);
    p
}

/// A grazer takes in: a mouthful of a living prey's tissue, drawn by what
/// each prey holds (ruling 287), digested and landed by TD5, and then, where
/// the prey's gland is charged by the ground under it (X5), a dose paid
/// from the reserve in the share of the prey the bite took.
pub(super) fn graze(i: u32, prey: u32) -> Process {
    let hand = tissue(prey);
    let eat = Effect::Eat {
        from: Binding::Target,
        amount: computed(mouthful(i)),
        into: hand.clone(),
        of: vec![hand.clone()],
    };
    let keep = |name: &str, value: Expr| Effect::Keep {
        name: name.into(),
        value,
    };
    let before = add(vec![
        held(Binding::Target, &hand),
        held(Binding::Actor, &hand),
    ]);
    let gland = || {
        Expr::Read(Reading::CellMass {
            who: Binding::Target,
            function: SECRETE.into(),
        })
    };
    let charged = mul(vec![
        at_least(gland(), c(1)),
        at_least(held(Binding::Place, SOIL), gland()),
    ]);
    let kept = |name: &str| Expr::Read(Reading::Kept { name: name.into() });
    let dose = div(
        mul(vec![gland(), kept(TAKEN)]),
        most(vec![c(1), kept(PREY)]),
    );
    let paid = Effect::Spend {
        from: vec![reserve(i)],
        to: Binding::Place,
        amount: computed(dose),
        into: Some(SOIL.into()),
    };
    let effects = [
        vec![
            eat,
            keep(TAKEN, held(Binding::Actor, &hand)),
            keep(PREY, before),
        ],
        landing(i, &hand, Conversion::Digestion),
        vec![Effect::When {
            guard: charged,
            then: vec![paid],
            otherwise: vec![],
        }],
    ]
    .concat();
    let mut p = due(&format!("body:graze-{i}"), Causation::Choice, effects, 2);
    p.requires.extend([
        own(i),
        expresses(INTAKE),
        Query::Computed(at_least(mouthful(i), c(1))),
    ]);
    p.target = Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: None,
        among: BTreeSet::from([format!("lineage:{prey}")]),
        weighted: true,
    });
    p
}

/// `gland.lua`, lowered (checkpoint 6's step 5d): once, four cells or five
/// of the frond by a draw, one where the ground cannot charge that many at
/// the frond's cell price, never more than the frond fixes with.
pub(super) fn gland(i: u32) -> Process {
    let part = |r: Reading| Expr::Read(r);
    let appetite = || add(vec![c(4), Expr::Draw { below: 2, slot: 0 }]);
    let price = mul(vec![
        part(Reading::CellWeight { who: Binding::Part }),
        appetite(),
    ]);
    let rich = at_least(held(Binding::Place, SOIL), price);
    let wanted = add(vec![c(1), mul(vec![rich, add(vec![appetite(), c(-1)])])]);
    let cells = |function: &str| {
        part(Reading::Cells {
            who: Binding::Part,
            function: function.into(),
        })
    };
    let develop = Effect::Allocate {
        from: Some(FIX.into()),
        to: SECRETE.into(),
        cells: computed(least(vec![wanted, cells(FIX)])),
    };
    let once = Effect::When {
        guard: at_least(c(0), cells(SECRETE)),
        then: vec![develop],
        otherwise: vec![],
    };
    let mut p = due(&format!("body:gland-{i}"), Causation::Choice, vec![once], 3);
    p.requires.extend([
        own(i),
        expresses(FIX),
        Query::Trait {
            who: Binding::Actor,
            key: CANDIDATE.into(),
        },
    ]);
    p
}

/// A body this light or lighter dies, its reserve returned to the site as
/// soil and its tissue kept as carrion.
fn starve(i: u32) -> Process {
    let release = Effect::Spend {
        from: vec![reserve(i)],
        to: Binding::Place,
        amount: computed(held(Binding::Actor, &reserve(i))),
        into: Some(SOIL.into()),
    };
    let effects = vec![release, Effect::Death];
    let mut p = due(
        &format!("body:starve-{i}"),
        Causation::Transition,
        effects,
        4,
    );
    p.requires.extend([
        own(i),
        Query::Below {
            who: Binding::Actor,
            key: tissue(i),
            amount: STARVATION_MG + 1,
        },
    ]);
    p
}

/// The site's mineralization rate (step 4): up to `dose` of the living
/// matter on the ground returned as soil a tick, a share of each.
pub(super) fn mineralize(lineages: u32, dose: u64) -> Process {
    let living = (0..lineages)
        .flat_map(|i| [tissue(i), reserve(i)])
        .collect();
    let effect = Effect::Convert {
        who: Binding::Place,
        from: living,
        to: SOIL.into(),
        amount: dose.into(),
        conversion: Conversion::Mineralization,
    };
    due("body:mineralize", Causation::Agentless, vec![effect], 5)
}

/// What lineage `i` does: keep its body and starve without it, and, as a
/// producer, fix and develop its gland, or, as a grazer, graze `prey`.
pub(super) fn natives(i: u32, prey: Option<u32>) -> Vec<Process> {
    let mut processes = vec![upkept(i), starve(i)];
    match prey {
        None => processes.extend([fix(i), gland(i)]),
        Some(prey) => processes.push(graze(i, prey)),
    }
    processes
}
