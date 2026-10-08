// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The natives that feed, by capability (rulings 569, 571, 575, 582, 588 to
//! 590): every lineage fixes and grazes, each only for a body whose systems
//! route the function, on its own lineage's accounts. Income is drawn as
//! far as fix's routes carry it; a bite is what intake's routes carry of the
//! mouthful, the limbs' share of it what the muscular routes carry to the
//! contracting parts, so a limbless grazer bites by its intake alone; what
//! cannot reach a part stays in the soil or the prey. The landing is then
//! bounded part by part by the carriage of what the act holds. A grazer
//! grazes any other lineage, and the dose is what the prey's glandular
//! routes carry to the part bitten. Each world draws its rates (571).

use super::physiology::*;
use crate::{rules::*, schema::*};
use std::collections::BTreeSet;

/// Ruling 505's units: income per voxel face of fixing area, and the
/// mouthful per voxel of intake before TD9's build multiple, each at a rate
/// a world draws over these denominators (571's reading, half to twice
/// 505's 11 and 12).
pub(super) const FACES: i64 = 144;
pub(super) const VOXELS: i64 = 269;

/// A world's feeding rates, numerators over `FACES` and `VOXELS`.
#[derive(Clone, Copy, Debug)]
pub(super) struct Rates {
    pub fixes: i64,
    pub grazes: i64,
}

const CONTRACT: &str = "function:contract";
pub(super) const INTAKE: &str = "function:intake";
const FIX: &str = "function:fix";
const SECRETE: &str = "function:secrete";
/// What a meal took and what its prey held before it, kept for the dose.
const TAKEN: &str = "probe:taken";
const PREY: &str = "probe:prey";

/// What a body's own matter lands in.
pub(super) fn lands(i: u32) -> Vec<Key> {
    vec![tissue(i), reserve(i), provision(i)]
}

/// What `who`'s systems naming `function` in `role` carry of `ask`.
pub(super) fn carried(
    who: Binding,
    (function, role): (&str, Role),
    ask: Expr,
    lands: Vec<Key>,
) -> Expr {
    joint(who, (function, role), ask, lands, None)
}

/// The same, joined to `joined` through the senses (659 to 664).
fn joint(
    who: Binding,
    (function, role): (&str, Role),
    ask: Expr,
    lands: Vec<Key>,
    joined: Option<(&str, Role)>,
) -> Expr {
    Expr::Carried {
        reading: Reading::Carried {
            who,
            function: function.into(),
            role,
            ask: 0,
            lands,
            joined: joined.map(|(f, r)| (f.into(), r)),
        },
        ask: Box::new(ask),
    }
}

pub(super) fn routes(who: Binding, function: &str, role: Role) -> Query {
    Query::Routes {
        who,
        function: function.into(),
        role,
    }
}

/// The landing's bounds from what the act holds of `hand` (581).
fn bounded(i: u32, who: Binding, (function, role): (&str, Role), hand: &str) -> Effect {
    Effect::Carry {
        who,
        function: function.into(),
        role,
        ask: computed(held(who, hand)),
        lands: lands(i),
    }
}

/// A producer's income (TD2c, ruling 505): the area its fixing presents at
/// the world's rate a face, at least a milligram, within its room.
fn income(i: u32, r: Rates) -> Expr {
    let rate = div(
        mul(vec![c(r.fixes), measured(FIX, Measure::Area)]),
        c(FACES),
    );
    least(vec![most(vec![c(1), rate]), room(i)])
}

/// TD9's mouthful (ruling 505): intake's volume at the world's rate a voxel,
/// and the build multiple's share as far as the muscular routes carry it
/// to the contracting parts (589) and the senses join them to intake (659 to
/// 664), within the room.
fn mouthful(i: u32, r: Rates) -> Expr {
    let volume = || measured(INTAKE, Measure::Volume);
    let base = div(mul(vec![c(r.grazes), volume()]), c(VOXELS));
    let priced = add(vec![ceiling(), mul(vec![span(), c(REFERENCE_MASS_MG)])]);
    let built = div(
        mul(vec![c(r.grazes), volume(), priced]),
        mul(vec![c(VOXELS), ceiling()]),
    );
    let share = joint(
        Binding::Actor,
        (CONTRACT, Role::Effect),
        less(built, base.clone()),
        vec![],
        Some((INTAKE, Role::Source)),
    );
    least(vec![most(vec![c(1), add(vec![base, share])]), room(i)])
}

/// What a bite takes: what intake's routes carry of the mouthful.
fn bite(i: u32, r: Rates) -> Expr {
    carried(
        Binding::Actor,
        (INTAKE, Role::Source),
        mouthful(i, r),
        lands(i),
    )
}

/// Lineage `i` fixes: its income drawn from the site's soil as far as its
/// routes carry it, shared out when the soil runs short (ruling 454),
/// synthesized and landed by TD5 within what reached each part.
pub(super) fn fix(i: u32, r: Rates) -> Process {
    let route = (FIX, Role::Source);
    let draw = Effect::Transfer {
        from: Binding::Place,
        to: Binding::Actor,
        account: SOIL.into(),
        amount: computed(carried(Binding::Actor, route, income(i, r), lands(i))),
    };
    let effects = [
        vec![draw, bounded(i, Binding::Actor, route, SOIL)],
        landing(i, SOIL, Conversion::Synthesis),
    ]
    .concat();
    let mut p = due(&format!("body:fix-{i}"), Causation::Choice, effects, 1);
    p.requires
        .extend([own(i), routes(Binding::Actor, FIX, Role::Source)]);
    p
}

/// Lineage `i` grazes lineage `prey`: a bite of a living prey's tissue,
/// drawn by what each prey holds (ruling 287), digested and landed by TD5,
/// and then, where the prey's gland is charged by the ground under it
/// (X5), a dose paid from the reserve in the share of the prey the bite
/// took, as far as the prey's glandular routes carry it to the part bitten.
pub(super) fn graze(i: u32, prey: u32, r: Rates) -> Process {
    let hand = tissue(prey);
    let eat = |whole| Effect::Eat {
        from: Binding::Target,
        amount: computed(bite(i, r)),
        into: hand.clone(),
        of: vec![hand.clone()],
        whole,
    };
    // A fed grazer takes a frond whole where its bite would take it all
    // (516); a hungry one eats it.
    let eat = Effect::When {
        guard: hungry(i),
        then: vec![eat(false)],
        otherwise: vec![eat(true)],
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
        amount: computed(carried(
            Binding::Target,
            (SECRETE, Role::Source),
            dose,
            vec![],
        )),
        into: Some(SOIL.into()),
    };
    let effects = [
        vec![
            eat,
            keep(TAKEN, held(Binding::Actor, &hand)),
            keep(PREY, before),
            bounded(i, Binding::Actor, (INTAKE, Role::Source), &hand),
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
        routes(Binding::Actor, INTAKE, Role::Source),
        Query::Computed(at_least(bite(i, r), c(1))),
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

/// What every lineage of `n` feeds by: fixing, and grazing each other
/// lineage. Where a lineage has one prey, as in the probe's two, its graze
/// keeps the lineage's name.
pub(super) fn natives(i: u32, n: u32, r: Rates) -> Vec<Process> {
    let others: Vec<u32> = (0..n).filter(|j| *j != i).collect();
    let grazes = others.iter().map(|j| {
        let mut p = graze(i, *j, r);
        if others.len() > 1 {
            p.id = format!("body:graze-{i}-{j}");
        }
        p
    });
    [fix(i, r)].into_iter().chain(grazes).collect()
}
