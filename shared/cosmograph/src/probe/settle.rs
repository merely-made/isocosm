// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 240: a world's competitions run at once in a tick, each against
//! its members' state at the tick's start, and settle at the tick's end.
//! What a member takes from each is gathered, and the acts that settle it
//! run in one order for both runners: every round's strain, then the reserve
//! it spent, summed and capped at what it holds, then what it won,
//! competition by competition in the order of what they contest.

use super::{
    Competition, Competitor,
    fight::{Act, Fight},
};
use crate::{meaning::value, schema::*};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

/// What one member takes from one competition in a tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Take {
    pub rounds: u64,
    pub spent: u64,
    pub gain: u64,
}

impl Take {
    pub fn gained(gain: u64) -> Self {
        Self {
            gain,
            ..Self::default()
        }
    }
    /// One side's take from a fight: the rounds it fought, the reserve it
    /// spent, and the ration if it won.
    pub fn fought(f: &Fight, side: usize, ration: u64) -> Self {
        let rounds = f.acts[side].iter().filter(|a| **a == Act::Round).count() as u64;
        Self {
            rounds,
            spent: f.acts[side].len() as u64 - rounds,
            gain: if f.winner == side { ration } else { 0 },
        }
    }
    pub fn is_nothing(&self) -> bool {
        *self == Self::default()
    }
}

/// The acts, by process, that settle what `member` took, in the order they
/// run. A lineage spends its reserve one way in every competition it enters
/// (checked at admission), so the cap falls the same whatever the order.
pub fn settlement(
    competitions: &BTreeMap<Key, Competition>,
    member: &Entity,
    takes: &BTreeMap<Key, Take>,
) -> Vec<Key> {
    fn own<'c>(c: &'c Competition, member: &Entity) -> Option<&'c Competitor> {
        c.kinds.iter().find(|k| member.traits.contains(&k.identity))
    }
    let mut acts = Vec::new();
    for (key, take) in takes {
        let round = &competitions[key].round;
        acts.extend(std::iter::repeat_n(round.clone(), take.rounds as usize));
    }
    let spent: u64 = takes.values().map(|t| t.spent).sum();
    let kind = takes.keys().find_map(|key| own(&competitions[key], member));
    if let Some(kind) = kind.filter(|_| spent > 0) {
        let held = value(&member.accounts, &kind.body);
        let paid = spent.min(held) as usize;
        acts.extend(std::iter::repeat_n(kind.spend.clone(), paid));
    }
    for (key, take) in takes {
        let c = &competitions[key];
        let Some(kind) = own(c, member) else {
            continue;
        };
        if take.gain == c.ration {
            acts.push(kind.eat.clone());
        } else if take.gain > 0 {
            acts.push(kind.share.clone());
        }
    }
    acts
}
