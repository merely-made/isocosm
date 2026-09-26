// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a probe world contests, and what each lineage does. Food feeds a
//! member's body, its reserve; water, when the domain has it, feeds a store
//! of its own, which dries a unit a tick. Each regrows from site soil, and
//! upkeep, drying and fight costs return matter to the soil, so it cycles.

use super::STRAIN;
use crate::{generate::process, rules::*, schema::*};
use std::collections::BTreeMap;

/// A thing a probe world contests (ruling 236): a site account regrown from
/// soil each tick, and the member account it feeds.
pub(super) struct Thing {
    /// The site account contested, which keys its competition.
    pub key: &'static str,
    /// The regrowing process's name.
    pub regrow: &'static str,
    /// The acts that take a full ration and half of one.
    pub take: &'static str,
    pub half: &'static str,
    /// Which of a lineage's accounts it feeds: 0 is the body.
    pub store: u32,
}

pub(super) const FOOD: Thing = Thing {
    key: "world:food",
    regrow: "probe:regrow",
    take: "eat",
    half: "share",
    store: 0,
};

pub(super) const WATER: Thing = Thing {
    key: "world:water",
    regrow: "probe:rain",
    take: "drink",
    half: "sip",
    store: 1,
};

pub(super) fn store(lineage: u32, store: u32) -> Key {
    format!("matter:{lineage}-{store}")
}

pub(super) fn account(key: &str, at_least: u64) -> Query {
    Query::Account {
        who: Binding::Actor,
        key: key.into(),
        at_least,
    }
}

pub(super) fn below(key: &str, amount: u64) -> Query {
    Query::Below {
        who: Binding::Actor,
        key: key.into(),
        amount,
    }
}

/// One unit of `from` returned to the site's soil.
fn spend(from: &str) -> Vec<Effect> {
    vec![
        Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::from([(from.into(), 1)]),
            give: BTreeMap::from([("world:soil".into(), 1)]),
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Place,
            account: "world:soil".into(),
            amount: 1,
        },
    ]
}

/// Strain taken in a fight; nothing when the draw gave none.
pub(super) fn strain(amount: u64) -> Vec<Effect> {
    if amount == 0 {
        return vec![];
    }
    vec![Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([(STRAIN.into(), amount)]),
    }]
}

/// `amount` of the contested thing taken from the site into `store`.
fn feed(thing: &Thing, store: &str, amount: u64) -> Vec<Effect> {
    vec![
        Effect::Transfer {
            from: Binding::Place,
            to: Binding::Actor,
            account: thing.key.into(),
            amount,
        },
        Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::from([(thing.key.into(), amount)]),
            give: BTreeMap::from([(store.into(), amount)]),
        },
    ]
}

/// The thing regrowing `amount` a tick at every site.
pub(super) fn regrow(thing: &Thing, amount: u64) -> Process {
    let mut p = process(
        thing.regrow,
        Shape::Agentless,
        vec![Effect::Transform {
            who: Binding::Place,
            take: BTreeMap::from([("world:soil".into(), amount)]),
            give: BTreeMap::from([(thing.key.into(), amount)]),
        }],
    );
    p.requires.push(Query::Account {
        who: Binding::Place,
        key: "world:soil".into(),
        at_least: amount,
    });
    p.period = Some(1);
    p.priority = -1;
    p
}

/// What one lineage does: keep its body, starve without it, spend it in
/// fights, and take each contested thing whole or halved, with the store
/// water fills drying a unit a tick.
pub(super) struct Acts {
    pub processes: Vec<Process>,
    pub spend: Key,
}

pub(super) fn lineage(i: u32, things: &[(&Thing, u64)], spend_strain: u64) -> Acts {
    let identity = format!("ability:probe-{i}");
    let body = store(i, 0);
    let own = Query::Trait {
        who: Binding::Actor,
        key: identity,
    };
    let mut upkeep = process(&format!("probe:upkeep-{i}"), Shape::Choice, spend(&body));
    upkeep.requires.extend([own.clone(), account(&body, 1)]);
    upkeep.period = Some(1);
    let mut starve = process(
        &format!("probe:starve-{i}"),
        Shape::Transition,
        vec![Effect::Death],
    );
    starve.requires.extend([own.clone(), below(&body, 1)]);
    starve.period = Some(1);
    starve.priority = 1;
    let mut fought = spend(&body);
    fought.extend(strain(spend_strain));
    let mut spent = process(&format!("probe:spend-{i}"), Shape::Choice, fought);
    spent.requires.extend([own.clone(), account(&body, 1)]);
    let spend_id = spent.id.clone();
    let mut processes = vec![upkeep, starve, spent];
    for &(thing, ration) in things {
        let fed = store(i, thing.store);
        for (name, amount) in [(thing.take, ration), (thing.half, ration / 2)] {
            let mut p = process(
                &format!("probe:{name}-{i}"),
                Shape::Choice,
                feed(thing, &fed, amount),
            );
            p.requires.push(own.clone());
            processes.push(p);
        }
        if thing.store > 0 {
            let mut dry = process(&format!("probe:dry-{i}"), Shape::Choice, spend(&fed));
            dry.requires.extend([own.clone(), account(&fed, 1)]);
            dry.period = Some(1);
            processes.push(dry);
        }
    }
    Acts {
        processes,
        spend: spend_id,
    }
}
