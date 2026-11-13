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

mod composition;
pub use composition::{Composition, Conversion};

/// Who holds an account a move leaves or reaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Holder {
    Site(Id),
    /// A member, and with a count above one the members after it.
    Entity(Id),
    /// A member's part, holding its own ledger (ruling 504), counted as its
    /// member is.
    Part(Id, crate::schema::PartId),
    /// The dev source (ruling 344): outside the world, holding nothing.
    Dev,
    ProjectReservation(crate::projects::ProjectId),
    ProjectConstruction(crate::projects::ProjectId),
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
    /// The lineage and kingdom of the body each side belongs to, as they
    /// stood when it moved; absent where a side is no body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_kind: Option<Kind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_kind: Option<Kind>,
    /// What the move carried, by material, where its maker says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition: Option<Composition>,
}

/// A body's lineage and kingdom, by key.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Kind {
    pub lineage: Key,
    pub kingdom: Key,
}

impl Holder {
    /// The body this holder is, or holds a part of.
    pub fn body(self) -> Option<Id> {
        match self {
            Holder::Entity(id) | Holder::Part(id, _) => Some(id),
            Holder::Site(_) | Holder::Dev | Holder::ProjectReservation(_) | Holder::ProjectConstruction(_) => None,
        }
    }
}

impl Flow {
    /// Whether the move stays inside one body.
    pub fn internal(&self) -> bool {
        matches!((self.from.0.body(), self.to.0.body()), (Some(a), Some(b)) if a == b)
    }

    /// Whether a process of this key made it.
    pub fn by_process(&self, key: &str) -> bool {
        matches!(&self.made_by, MadeBy::Process(k) if k == key)
    }
}

/// What the dev source issued over `flows`: every move out of it.
pub fn issued(flows: &[Flow]) -> u64 {
    let out = flows.iter().filter(|f| f.from.0 == Holder::Dev);
    out.map(|f| f.amount.saturating_mul(f.count)).sum()
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
        self.sim.flows.harms.clear();
        let result = run(self);
        let taken = std::mem::take(&mut self.sim.flows);
        // The harms stay readable, by `Simulation::harms`, until the next record.
        self.sim.flows.harms = taken.harms;
        let flows = taken.flows;
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

/// A surface's move and the kind each side stands for (cases 1223, 1225, 1226).
pub(crate) struct Kinded {
    pub made_by: MadeBy,
    pub leg: Leg,
    pub kinds: [Option<Kind>; 2],
}

/// Where one take or give of a body's own matter went among its parts
/// (ruling 504), in the order the act made it.
#[derive(Clone, Debug)]
pub(crate) struct Routed {
    pub body: Id,
    pub key: Key,
    pub give: bool,
    pub parts: Vec<(crate::schema::PartId, u64)>,
}

/// An act's legs with each side that moved a body's own matter split among
/// the parts it left or reached, in order; a side nothing routed keeps its
/// holder.
pub(crate) fn split(legs: Vec<Leg>, routed: Vec<Routed>) -> Vec<Leg> {
    use std::collections::{BTreeMap, VecDeque};
    if routed.is_empty() {
        return legs;
    }
    let mut queues: BTreeMap<(Id, Key, bool), VecDeque<(crate::schema::PartId, u64)>> =
        BTreeMap::new();
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

/// A bite that harmed its eater (the spoilage rule, lineage-rounds D37):
/// the meal happened and its matter moved, and the eater lost `cells` to a
/// wound as a hazard's hit takes them. Handed over beside the flows, and
/// like them outside the world's state, hash and save.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Harm {
    pub tick: Tick,
    /// The process whose meal it was.
    pub made_by: Key,
    pub eater: Id,
    pub prey: Id,
    /// The age of the prey's flesh when it was dead; fresh while it lived.
    pub flesh: crate::ecology::Flesh,
    /// The matter that harmed, the prey's lineage.
    pub lineage: Key,
    pub cells: u32,
    /// The place it was eaten at.
    pub place: Id,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Flows {
    keeping: bool,
    flows: Vec<Flow>,
    /// The flows kept when the advance under way began.
    kept: usize,
    harms: Vec<Harm>,
    kept_harms: usize,
}

impl Simulation {
    pub(crate) fn flowing(&self) -> bool {
        self.flows.keeping
    }

    /// The bites that harmed their eaters in the last recorded tick or
    /// command (`advance_tick_with_flows`, `command_with_flows`).
    #[must_use]
    pub fn harms(&self) -> &[Harm] {
        &self.flows.harms
    }

    /// Records an accepted act's harms.
    pub(crate) fn harmed(&mut self, harms: Vec<Harm>) {
        if self.flows.keeping {
            self.flows.harms.extend(harms);
        }
    }

    /// Records an accepted act's moves, standing for `count` members.
    pub(crate) fn flowed(&mut self, made_by: MadeBy, legs: Vec<Leg>, count: u64) {
        if !self.flows.keeping {
            return;
        }
        let tick = self.state.tick;
        let population = &self.state.population;
        let kind = |h: Holder| {
            let e = population.get(h.body()?)?;
            Some(Kind {
                lineage: e.lineage.clone(),
                kingdom: e.kingdom.clone(),
            })
        };
        self.flows.flows.extend(legs.into_iter().map(|leg| Flow {
            tick,
            made_by: made_by.clone(),
            from_kind: kind(leg.from.0),
            to_kind: kind(leg.to.0),
            from: leg.from,
            to: leg.to,
            amount: leg.amount,
            count,
            composition: None,
        }));
    }

    /// Records a surface's moves (cases 1223, 1225, 1226): its stock is no
    /// body's, so each leg names the lineage and kingdom of each side itself.
    pub(crate) fn flowed_kinds(&mut self, moves: Vec<Kinded>) {
        if !self.flows.keeping {
            return;
        }
        let tick = self.state.tick;
        self.flows.flows.extend(moves.into_iter().map(|m| Flow {
            tick,
            made_by: m.made_by,
            from_kind: m.kinds[0].clone(),
            to_kind: m.kinds[1].clone(),
            from: m.leg.from,
            to: m.leg.to,
            amount: m.leg.amount,
            count: 1,
            composition: None,
        }));
    }

    /// Marks where an advance begins, to put its moves back if it is refused.
    pub(crate) fn flows_from(&mut self) {
        self.flows.kept = self.flows.flows.len();
        self.flows.kept_harms = self.flows.harms.len();
    }

    pub(crate) fn unflow(&mut self) {
        self.flows.flows.truncate(self.flows.kept);
        self.flows.harms.truncate(self.flows.kept_harms);
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
