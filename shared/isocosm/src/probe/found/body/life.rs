// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Births and care in the probe (rulings 514, 519, 521, 524 and 526 to
//! 529). Each when its provision is full, a producer buds and a grazer
//! broods when fed or lays its clutch of eggs when hungry, a semelparous
//! cohort dying as its birth is done. A fed grazer with something
//! provisioned nurses a hungry young of its own with milk, its refilling
//! provision going to the young rather than to its next birth.

use super::physiology::*;
use crate::{bodied::SEMELPAROUS, rules::*};
use std::collections::BTreeSet;

/// The mark a producer's bud carries until it severs (524).
pub(super) const BUD_MARK: &str = "part:bud";
/// The cohort trait the probe draws beside semelparity (529).
pub(super) const ITEROPAROUS: &str = "life:iteroparous";

/// Its provision full, and something in it.
fn provisioned(i: u32) -> Query {
    let full = at_least(c(0), provision_gap(i));
    let some = at_least(held(Binding::Actor, &provision(i)), c(1));
    Query::Computed(mul(vec![full, some]))
}

fn cohort(once: bool) -> Query {
    Query::Trait {
        who: Binding::Actor,
        key: if once { SEMELPAROUS } else { ITEROPAROUS }.into(),
    }
}

fn named(what: &str, i: u32, once: bool) -> String {
    match once {
        true => format!("body:{what}-once-{i}"),
        false => format!("body:{what}-{i}"),
    }
}

/// A producer's bud (524), the semelparous parent dying as it severs.
fn bud(i: u32, once: bool) -> Process {
    let effect = Effect::Bud {
        mark: BUD_MARK.into(),
        once,
    };
    let mut p = due(&named("bud", i, once), Causation::Choice, vec![effect], 6);
    p.requires.extend([own(i), cohort(once), provisioned(i)]);
    p
}

/// A grazer's birth: a clutch of eggs while hungry, a brood while fed
/// (519), the semelparous parent dying when it is done.
fn bear(i: u32, once: bool) -> Process {
    let birth = Effect::When {
        guard: hungry(i),
        then: vec![Effect::Bear { clutch: true }],
        otherwise: vec![Effect::Bear { clutch: false }],
    };
    let effects = match once {
        true => vec![birth, Effect::Death],
        false => vec![birth],
    };
    let mut p = due(&named("bear", i, once), Causation::Choice, effects, 6);
    p.requires.extend([own(i), cohort(once), provisioned(i)]);
    p
}

/// Milk (527, 528): what the fed parent has provisioned, as much as its
/// hungry young has room for, moved to the young and digested into its
/// tissue.
fn nurse(i: u32) -> Process {
    let kept = || {
        Expr::Read(Reading::Kept {
            name: "milk".into(),
        })
    };
    let room = most(vec![
        c(0),
        less(
            ceiling_of(Binding::Target),
            held(Binding::Target, &tissue(i)),
        ),
    ]);
    let milk = least(vec![held(Binding::Actor, &provision(i)), room]);
    let effects = vec![
        Effect::Keep {
            name: "milk".into(),
            value: milk,
        },
        Effect::Transfer {
            from: Binding::Actor,
            to: Binding::Target,
            account: provision(i),
            amount: computed(kept()),
        },
        Effect::Convert {
            who: Binding::Target,
            from: vec![provision(i)],
            to: tissue(i),
            amount: computed(kept()),
            conversion: Conversion::Digestion,
        },
    ];
    let mut p = due(&format!("body:nurse-{i}"), Causation::Choice, effects, 5);
    let fed = less(c(1), hungry(i));
    let something = at_least(held(Binding::Actor, &provision(i)), c(1));
    p.requires.extend([
        own(i),
        Query::Related {
            kind: "sim:child".into(),
        },
        Query::Computed(mul(vec![fed, something, hungry_of(i, Binding::Target)])),
    ]);
    p.target = Some(Target {
        same_place: true,
        alive: Some(true),
        lineage: Some(format!("lineage:{i}")),
        among: BTreeSet::new(),
        weighted: false,
    });
    p
}

/// A producer buds; a grazer bears and nurses.
pub(super) fn natives(i: u32, producer: bool) -> Vec<Process> {
    match producer {
        true => vec![bud(i, false), bud(i, true)],
        false => vec![bear(i, false), bear(i, true), nurse(i)],
    }
}
