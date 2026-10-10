// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The deliberative methodology (ruling 683): a deliberative critter takes
//! one Choice process of those due, weighing its needs, its mood, its live
//! nudges and the bond each came by; its receipt's `foregone` lists the rest.
//! An answered nudge moves its bond by how the act served the critter (688).

use super::{Aim, Answer, Nudge, Toward};
use crate::{
    meaning::{Named, Scene},
    rules::*,
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

/// Each deliberative critter's choice at one tick, kept for an advance.
#[derive(Clone, Debug, Default)]
pub(crate) struct Chosen {
    tick: Tick,
    by: BTreeMap<Id, Choice>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Choice {
    pub process: Option<Key>,
    pub target: Option<Id>,
    /// The live nudges, by index, that the chosen act answers.
    pub answers: Vec<usize>,
    /// What each considered process scored, and on what target.
    pub options: BTreeMap<Key, (i64, Option<Id>)>,
}

/// What the methodology says of one actor's act in a pass.
pub(crate) enum Deliberated {
    /// Not a deliberative choice: the act runs as any other.
    Free,
    /// The critter chose another act this tick.
    Foregone,
    Chosen {
        target: Option<Id>,
        answers: Vec<usize>,
    },
}

impl Deliberated {
    pub(crate) fn target(&self) -> Option<Id> {
        match self {
            Self::Chosen { target, .. } => *target,
            _ => None,
        }
    }
}

/// The account a need reads, where it reads one of the actor's.
fn need_account(q: &Query) -> Option<&Key> {
    match q {
        Query::Below {
            who: Binding::Actor,
            key,
            ..
        } => Some(key),
        _ => None,
    }
}

/// Whether `e` gives the actor `key`, or any matter where `key` is none.
fn feeds(e: &Effect, key: Option<&Key>) -> bool {
    let is = |k: &Key| key.is_none_or(|key| key == k);
    match e {
        Effect::Transform {
            who: Binding::Actor,
            give,
            ..
        } => give.keys().any(is),
        Effect::Transfer {
            to: Binding::Actor,
            account,
            ..
        } => is(account),
        Effect::Eat { into, .. } => is(into),
        Effect::Convert {
            who: Binding::Actor,
            to,
            ..
        } => is(to),
        Effect::When { .. } => e.branches().any(|b| feeds(b, key)),
        _ => false,
    }
}

/// Whether `p` acts on its site's ground or conditions.
fn binds_place(p: &Process) -> bool {
    fn at(e: &Effect) -> bool {
        let place = |b: &Binding| *b == Binding::Place;
        match e {
            Effect::Transfer { from, to, .. } => place(from) || place(to),
            Effect::Transform { who, .. } | Effect::Convert { who, .. } => place(who),
            Effect::Eat { from, .. } => place(from),
            Effect::Condition { .. } | Effect::Spend { .. } | Effect::Grow { .. } => true,
            Effect::When { .. } => e.branches().any(at),
            _ => false,
        }
    }
    p.commitments.iter().chain(&p.effects).any(at)
}

/// The sites `p` moves its actor to.
pub(crate) fn moves(p: &Process) -> Vec<Id> {
    let to = |e: &Effect| match e {
        Effect::Move { destination } => Some(*destination),
        _ => None,
    };
    p.commitments
        .iter()
        .chain(&p.effects)
        .filter_map(to)
        .collect()
}

impl Simulation {
    /// Whether `actor` takes `process` now, by its methodology (683).
    pub(crate) fn deliberate(&mut self, actor: Id, process: &Process) -> Deliberated {
        let deliberative = self
            .body_at_start(actor)
            .is_some_and(|e| e.method == Method::Deliberative);
        if !deliberative || process.causation != Causation::Choice {
            return Deliberated::Free;
        }
        let tick = self.state.tick;
        if self.chosen.tick != tick {
            self.chosen = Chosen {
                tick,
                by: BTreeMap::new(),
            };
        }
        if !self.chosen.by.contains_key(&actor) {
            let choice = self.choose(actor);
            self.chosen.by.insert(actor, choice);
        }
        let c = &self.chosen.by[&actor];
        match c.process.as_deref() == Some(process.id.as_str()) {
            true => Deliberated::Chosen {
                target: c.target,
                answers: c.answers.clone(),
            },
            false => Deliberated::Foregone,
        }
    }

    /// The live mood of `actor` and the weights of the needs pressing it.
    pub(crate) fn minded(&self, actor: Id) -> (i64, Vec<&Need>) {
        let Some(e) = self.state.population.get(actor) else {
            return (0, vec![]);
        };
        let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
        let scene = Scene {
            actor: Some(e),
            target: Named::Unnamed,
            part: None,
            site: self.state.sites.get(&e.place),
            tick: self.state.tick,
            related: &related,
            rules: &self.genesis.rules,
            lineages: Some(&self.state.lineages),
        };
        let pressing: Vec<&Need> = crate::meaning::needs(&self.genesis.rules)
            .iter()
            .filter(|n| n.traits.iter().all(|t| e.traits.contains(t)))
            .filter(|n| crate::meaning::read(&n.query, &scene).is_ok_and(|r| r.0))
            .collect();
        (pressing.iter().map(|n| n.weight).sum(), pressing)
    }

    /// What the actor holds and how it feels: what an answered act is
    /// weighed by.
    pub(crate) fn wellbeing(&self, actor: Id) -> (i64, u128) {
        let held = self.state.population.get(actor).map_or(0, |e| {
            crate::meaning::mass(&crate::anatomy::books(e), &self.genesis.rules)
        });
        (self.minded(actor).0, held)
    }

    fn feasible(&self, actor: Id, target: Option<Id>, p: &Process, place: Id) -> bool {
        let part = self.bind_part(actor, p);
        self.target_matches(actor, target, p)
            && p.requires
                .iter()
                .all(|q| self.query(actor, target, place, part, q).is_ok())
    }

    /// Whether an act of `p` at `place` on `target` answers `n`.
    fn answers(&self, n: &Nudge, p: &Process, place: Id, target: Option<Id>) -> bool {
        if n.act.as_ref().is_some_and(|act| *act != p.id) {
            return false;
        }
        let site = match n.toward {
            Toward::Thing(t) if n.aim == Aim::Act => return target == Some(t),
            Toward::Thing(t) => match self.state.population.get(t) {
                Some(e) => e.place,
                None => return false,
            },
            Toward::Site(s) => s,
        };
        let goes = moves(p);
        if place != site {
            let hops = |from| super::tier::hops(&self.state.sites, from, site);
            let here = hops(place);
            return goes
                .iter()
                .any(|d| hops(*d).is_some_and(|h| here.is_none_or(|x| h < x)));
        }
        goes.is_empty() && (n.aim == Aim::Attend || binds_place(p))
    }

    /// The act `actor` takes this tick: the due Choice process it scores
    /// highest, the first in identity order among equals.
    pub(crate) fn choose(&self, actor: Id) -> Choice {
        self.consider(actor, true)
    }

    /// What `actor` weighs: the Choice processes due now, or all of them
    /// where `due_only` is false, as readings ask.
    pub(crate) fn consider(&self, actor: Id, due_only: bool) -> Choice {
        let tick = self.state.tick;
        let rules = &self.genesis.rules;
        let directing = rules.directing();
        let Some(place) = self.body_at_start(actor).map(|e| e.place) else {
            return Choice::default();
        };
        let live = self.live_nudges(actor, tick);
        let (mood, pressing) = self.minded(actor);
        let mut best = Choice::default();
        let mut top = i64::MIN;
        let due = |p: &&Process| p.period.is_some_and(|n| tick % n == 0);
        let choices = rules
            .processes
            .values()
            .filter(|p| p.causation == Causation::Choice);
        for p in choices.filter(|p| !due_only || due(p)) {
            let have = |a: &Key| {
                self.body_at_start(actor)
                    .map_or(0, |e| e.accounts.get(a).copied().unwrap_or(0))
            };
            if p.need_account
                .as_ref()
                .is_some_and(|a| have(a) >= p.need_below)
            {
                continue;
            }
            // A nudge to act on a thing names the target, where it can be.
            let nudged = live.iter().find_map(|&i| match self.state.nudges[i] {
                Nudge {
                    aim: Aim::Act,
                    toward: Toward::Thing(t),
                    ..
                } if p.target.is_some() && self.feasible(actor, Some(t), p, place) => Some(t),
                _ => None,
            });
            let target = match p.target {
                Some(_) => nudged.or_else(|| self.choose_target(actor, p)),
                None => None,
            };
            if !self.feasible(actor, target, p, place) {
                continue;
            }
            let effects = || p.commitments.iter().chain(&p.effects);
            let needs: i64 = pressing
                .iter()
                .filter(|n| effects().any(|e| feeds(e, need_account(&n.query))))
                .map(|n| -n.weight.min(0))
                .sum();
            // Low mood weighs needs harder.
            let needs = needs * 100 * (100 + (-mood).max(0)) / 100;
            let answers: Vec<usize> = live
                .iter()
                .copied()
                .filter(|&i| self.answers(&self.state.nudges[i], p, place, target))
                .collect();
            let sway: i64 = answers
                .iter()
                .map(|&i| {
                    let n = &self.state.nudges[i];
                    let bond = self.bond(n.participant, actor).unwrap_or(0);
                    directing.sway * bond / directing.most
                })
                .sum();
            let score = needs + sway + i64::from(p.priority);
            best.options.insert(p.id.clone(), (score, target));
            if score > top {
                top = score;
                best.process = Some(p.id.clone());
                best.target = target;
                best.answers = answers;
            }
        }
        best
    }

    /// Moves the bond of each nudge an act answered, a step up where it
    /// served the critter and down where it did not, and marks it answered.
    pub(crate) fn answered(
        &mut self,
        actor: Id,
        process: &str,
        answers: &[usize],
        (mood, held): (i64, u128),
        accepted: bool,
    ) {
        let (now, has) = self.wellbeing(actor);
        let served = accepted && (now > mood || (now == mood && has >= held));
        let d = self.genesis.rules.directing();
        let site = self
            .state
            .population
            .get(actor)
            .map_or(super::PLACELESS, |e| e.place);
        if let Some(j) = &mut self.journal {
            j.nudges(&self.state.nudges);
        }
        for &i in answers {
            let participant = self.state.nudges[i].participant;
            let bond = self.bond(participant, actor).unwrap_or(0);
            let step = if served { d.step } else { -d.step };
            self.set_bond(participant, actor, (bond + step).clamp(0, d.most));
            self.state.nudges[i].answer = Some(Answer {
                tick: self.state.tick,
                process: process.into(),
                site,
                served,
            });
        }
    }
}
