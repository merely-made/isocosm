// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The flow record (rulings 270, 345 and 359): every matter move an accepted
//! act makes, from holder and account to holder and account, with what each
//! member moved, the members the act stood for, and what made the move. A
//! host asks for it, as for the watch; it is kept outside the world's state,
//! so it never reaches a hash or a save, and handed over when taken. An act
//! that is not accepted leaves no flow, and an advance put back takes its
//! flows with it.

use crate::{
    rules::{AccountKind, Binding, Effect, Rules},
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};

/// Who holds an account a move leaves or reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Holder {
    Site(Id),
    /// A member, and with a count above one the members after it.
    Entity(Id),
    /// The dev source (ruling 344): outside the world, holding nothing.
    Dev,
}

/// What made a move: an act of a process, or a host command, which is how
/// the dev source's placements are marked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MadeBy {
    Process(Key),
    /// The command's name as the history writes it, `PlaceMatter`.
    Command(Key),
}

/// One matter move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flow {
    pub tick: Tick,
    pub made_by: MadeBy,
    pub from: (Holder, Key),
    pub to: (Holder, Key),
    /// What each member moved.
    pub amount: u64,
    /// The members the move stands for: an entity holder and those after
    /// it, each moving `amount`; a site gives or takes `amount` for each.
    pub count: u64,
}

/// One move an act stages, before it is known to be accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Leg {
    pub from: (Holder, Key),
    pub to: (Holder, Key),
    pub amount: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Flows {
    keeping: bool,
    flows: Vec<Flow>,
    /// The flows kept when the advance under way began.
    kept: usize,
}

impl Simulation {
    /// Keeps every matter move from now on, until taken.
    pub fn keep_flows(&mut self) {
        self.flows.keeping = true;
    }

    /// The moves kept since the last call, in the order they were made.
    pub fn take_flows(&mut self) -> Vec<Flow> {
        self.flows.kept = 0;
        std::mem::take(&mut self.flows.flows)
    }

    pub(crate) fn flowing(&self) -> bool {
        self.flows.keeping
    }

    /// Records an accepted act's moves, standing for `count` members.
    pub(crate) fn flowed(&mut self, made_by: MadeBy, legs: Vec<Leg>, count: u64) {
        if !self.flows.keeping {
            return;
        }
        let tick = self.state.tick;
        self.flows.flows.extend(legs.into_iter().map(|leg| Flow {
            tick,
            made_by: made_by.clone(),
            from: leg.from,
            to: leg.to,
            amount: leg.amount,
            count,
        }));
    }

    /// Marks where an advance begins, to put its moves back if it is refused.
    pub(crate) fn flows_from(&mut self) {
        self.flows.kept = self.flows.flows.len();
    }

    pub(crate) fn unflow(&mut self) {
        self.flows.flows.truncate(self.flows.kept);
    }
}

/// The matter moves a transfer or transform makes, read from its definition,
/// `holder` naming who holds each binding's ledger in the act.
pub(crate) fn moves(
    e: &Effect,
    rules: &Rules,
    holder: impl Fn(Binding) -> Option<Holder>,
) -> Vec<Leg> {
    let matter = |k: &Key| matches!(rules.accounts.get(k), Some(AccountKind::Matter { .. }));
    let held = |l: &Ledger| -> Vec<(Key, u64)> {
        let moved = l.iter().filter(|(k, v)| matter(k) && **v > 0);
        moved.map(|(k, v)| (k.clone(), *v)).collect()
    };
    match e {
        Effect::Transfer {
            from,
            to,
            account,
            amount,
        } if matter(account) && *amount > 0 => match (holder(*from), holder(*to)) {
            (Some(f), Some(t)) if f != t => vec![Leg {
                from: (f, account.clone()),
                to: (t, account.clone()),
                amount: *amount,
            }],
            _ => vec![],
        },
        Effect::Transform {
            who, take, give, ..
        } => holder(*who).map_or(vec![], |h| poured(h, held(take), held(give))),
        _ => vec![],
    }
}

/// A transform's matter moves within one holder: what it takes poured into
/// what it gives, both in key order. Admission balances the two, so every
/// unit taken is given.
pub(crate) fn poured(
    holder: Holder,
    take: impl IntoIterator<Item = (Key, u64)>,
    give: impl IntoIterator<Item = (Key, u64)>,
) -> Vec<Leg> {
    let mut gives = give.into_iter().filter(|(_, v)| *v > 0);
    let mut left_in_give = 0;
    let mut give_key = Key::new();
    let mut legs = Vec::new();
    for (take_key, amount) in take {
        let mut left = amount;
        while left > 0 {
            if left_in_give == 0 {
                let Some((k, v)) = gives.next() else {
                    return legs;
                };
                (give_key, left_in_give) = (k, v);
            }
            let moved = left.min(left_in_give);
            if take_key != give_key {
                legs.push(Leg {
                    from: (holder, take_key.clone()),
                    to: (holder, give_key.clone()),
                    amount: moved,
                });
            }
            left -= moved;
            left_in_give -= moved;
        }
    }
    legs
}
