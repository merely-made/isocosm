// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The flow record (rulings 270, 345 and 359): every matter move an accepted
//! act makes, from holder and account to holder and account, with what each
//! member moved, the members the act stood for, and what made the move. A
//! host asks for one tick or one command (ruling 371); its moves are handed
//! over in that call's result, outside the world's state, hash and save.
//! Recording never stays on between calls. An act that is not accepted
//! leaves no flow, and a tick put back hands over no result.

use crate::{
    Result,
    history::{Command, Session},
    rules::{AccountKind, Binding, Effect, Rules},
    schema::*,
    simulation::{Simulation, Work},
};
use serde::{Deserialize, Serialize};

/// Who holds an account a move leaves or reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Holder {
    Site(Id),
    /// A member, and with a count above one the members after it.
    Entity(Id),
    /// A member's part, holding its own ledger (ruling 504), counted as its
    /// member is.
    Part(Id, Id),
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

/// One completed tick or command and every matter move it made. Commands
/// remain at their current tick; an advance returns the tick it completed.
/// The caller owns the moves: retaining this result cannot make a later
/// result repeat them. An empty `flows` still identifies the completed tick.
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub struct FlowResult<T> {
    pub tick: Tick,
    pub result: T,
    pub flows: Vec<Flow>,
}

impl Session {
    /// Advances exactly one tick and returns its matter moves, when asked.
    /// The existing one-tick transaction and work budget apply: an error
    /// rolls back the tick and returns no record. For several recorded ticks,
    /// call again after handling each result; each call is its own transaction.
    /// Ordinary `advance(ticks)` remains unrecorded and atomic over its span.
    pub fn advance_tick_with_flows(&mut self) -> Result<FlowResult<Work>> {
        self.with_flows(|session| session.advance(1))
    }

    /// Runs a host command and hands over its moves at the current tick.
    /// Several commands at one tick yield separate, non-overlapping results.
    /// A refused act has an outcome but no moves; a failed command returns
    /// its existing error. Neither recording nor moves survive the call.
    pub fn command_with_flows(&mut self, command: Command) -> Result<FlowResult<String>> {
        self.with_flows(|session| session.command(command))
    }

    fn with_flows<T>(&mut self, run: impl FnOnce(&mut Self) -> Result<T>) -> Result<FlowResult<T>> {
        self.sim.flows.keeping = true;
        let result = run(self);
        let flows = std::mem::take(&mut self.sim.flows).flows;
        result.map(|result| FlowResult {
            tick: self.sim.state.tick,
            result,
            flows,
        })
    }
}

/// One move an act stages, before it is known to be accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Leg {
    pub from: (Holder, Key),
    pub to: (Holder, Key),
    pub amount: u64,
}

/// Where one take or give of a body's own matter went among its parts
/// (ruling 504), in the order the act made it.
#[derive(Clone, Debug)]
pub(crate) struct Routed {
    pub body: Id,
    pub key: Key,
    pub give: bool,
    pub parts: Vec<(Id, u64)>,
}

/// An act's legs with each side that moved a body's own matter split among
/// the parts it left or reached, in order; a side nothing routed keeps its
/// holder.
pub(crate) fn split(legs: Vec<Leg>, routed: Vec<Routed>) -> Vec<Leg> {
    use std::collections::{BTreeMap, VecDeque};
    if routed.is_empty() {
        return legs;
    }
    let mut queues: BTreeMap<(Id, Key, bool), VecDeque<(Id, u64)>> = BTreeMap::new();
    for r in routed {
        queues
            .entry((r.body, r.key, r.give))
            .or_default()
            .extend(r.parts);
    }
    let mut side = |(holder, key): &(Holder, Key), amount: u64, give: bool| {
        let Holder::Entity(body) = *holder else {
            return vec![(*holder, amount)];
        };
        let Some(queue) = queues.get_mut(&(body, key.clone(), give)) else {
            return vec![(*holder, amount)];
        };
        let (mut left, mut pieces) = (amount, vec![]);
        while left > 0 {
            let Some((part, held)) = queue.front_mut() else {
                pieces.push((*holder, left));
                break;
            };
            let moved = left.min(*held);
            pieces.push((Holder::Part(body, *part), moved));
            (*held, left) = (*held - moved, left - moved);
            if *held == 0 {
                queue.pop_front();
            }
        }
        pieces
    };
    let mut out = vec![];
    for leg in legs {
        let from = side(&leg.from, leg.amount, false);
        let mut to = side(&leg.to, leg.amount, true).into_iter();
        let mut current = to.next();
        for (holder, mut amount) in from {
            while amount > 0 {
                let Some((into, room)) = current.as_mut() else {
                    break;
                };
                let moved = amount.min(*room);
                out.push(Leg {
                    from: (holder, leg.from.1.clone()),
                    to: (*into, leg.to.1.clone()),
                    amount: moved,
                });
                amount -= moved;
                *room -= moved;
                if *room == 0 {
                    current = to.next();
                }
            }
        }
    }
    out
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Flows {
    keeping: bool,
    flows: Vec<Flow>,
    /// The flows kept when the advance under way began.
    kept: usize,
}

impl Simulation {
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
        // An act resolves its amounts before its moves are worked out.
        Effect::Transfer {
            from,
            to,
            account,
            amount,
        } if matter(account) => match (holder(*from), holder(*to), amount.resolved()) {
            (Some(f), Some(t), Ok(amount)) if f != t && amount > 0 => vec![Leg {
                from: (f, account.clone()),
                to: (t, account.clone()),
                amount,
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
