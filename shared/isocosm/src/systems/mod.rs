// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body's systems (rulings 560 to 590): which it carries, which of them
//! route a native's function, and what they carry part by part. A role is
//! filled by the living parts expressing a function it names, by every
//! living part, or by the part a bite landed on; a system is realized where
//! a living part fills its sources and one fills its effects (574's
//! reading), stores and gates joining a route where a part expresses them.

mod carry;
mod vary;

pub use carry::{Carried, capacity, carry};
pub use vary::{Riff, founded, inherit, regrow, riff, take_up, vary};

use crate::{
    rules::{Fill, Role, System},
    schema::*,
};
use std::collections::BTreeSet;

/// Whether a living part fills `fill`.
fn fills(p: &Part, fill: &Fill, bitten: bool) -> bool {
    match fill {
        Fill::Function(f) => p.functions.contains(f),
        Fill::Living => true,
        Fill::Bitten => bitten,
    }
}

/// The living parts filling any of `role`; `bitten` the part a bite landed
/// on, if one did.
pub fn filling(e: &Entity, role: &BTreeSet<Fill>, bitten: Option<Id>) -> BTreeSet<Id> {
    let living = e.parts.iter().filter(|(_, p)| !p.severed);
    living
        .filter(|(id, p)| role.iter().any(|f| fills(p, f, bitten == Some(**id))))
        .map(|(id, _)| *id)
        .collect()
}

/// Whether `e` realizes `s`: a living part fills its sources and one its
/// effects, the parts a bite lands on being any living part.
pub fn realizes(e: &Entity, s: &System) -> bool {
    let filled = |role: &BTreeSet<Fill>| {
        let living = e.parts.values().filter(|p| !p.severed);
        living
            .flat_map(|p| role.iter().map(move |f| fills(p, f, true)))
            .any(|v| v)
    };
    filled(&s.sources) && filled(&s.effects)
}

/// The systems `e` carries that name `function` in `role`, as one network:
/// all their sources, stores, gates and effects (582). `None` where none
/// names it.
pub fn union(e: &Entity, function: &str, role: Role) -> Option<System> {
    let mut naming = e.systems.values().filter(|s| s.routes(function, role));
    let first = naming.next()?.clone();
    Some(naming.fold(first, |mut u, s| {
        for r in Role::ALL {
            u.role_mut(r).extend(s.role(r).iter().cloned());
        }
        u
    }))
}

/// Whether `e`'s systems route `function` in `role` (575): one it carries
/// names it there, a living part expresses it, and the systems naming it
/// are realized as one.
pub fn routes(e: &Entity, function: &str, role: Role) -> bool {
    let expressed = e
        .parts
        .values()
        .any(|p| !p.severed && p.functions.contains(function));
    expressed && union(e, function, role).is_some_and(|u| realizes(e, &u))
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod vary_tests;
