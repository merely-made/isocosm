// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Members' learning (cases 1051, 1053 and 1058; mesocosm-overlay M3.6). A
//! member that chose an act by its lineage's policy reads, once the act is
//! done, whether it served it, as `answered` reads a nudged act, and moves
//! its own delta on the needs that were pressing when it chose: up if
//! served, down if not. The delta is added to the drives the member's
//! policy is expressed with, so two members of one lineage with different
//! histories choose differently. It fades by whole spans, never by a
//! per-tick process, and a third of it passes to each child (case 1058).
//! Only one parent is visible at a birth, so a child takes from that one.
//!
//! A member also weighs each kind of act (`Learned.acts`), which scores an
//! act the needs do not, as a move. An act that ends without bettering the
//! member becomes its pending act; a later act that betters it within the
//! rule's `horizon` credits that one as well, and one that waits out the
//! horizon is marked down.

use super::choice::Choice;
use crate::{
    rules::{Learning, Need, Rules},
    schema::*,
    simulation::Simulation,
};
use std::sync::Arc;

/// The world's learning rule, if it keeps one.
pub(crate) fn rule(rules: &Rules) -> Option<Learning> {
    rules.directing.as_ref()?.learning
}

/// What a child born at `tick` carries of its parent's delta (case 1058);
/// none where the world keeps no learning.
pub(crate) fn inherited(parent: &Option<Learned>, rules: &Rules, tick: Tick) -> Option<Learned> {
    rule(rules).and_then(|l| l.handed(parent.as_ref()?, tick))
}

/// The ticks of life before a member of `e`'s lineage is mature: the least
/// age gate on a process that bears young for a body of its traits, which
/// is where the ecology's generators put maturity (the `Query::Age` of its
/// birth process). None where nothing bears, and then nothing is young.
pub(crate) fn maturity(rules: &Rules, e: &Entity) -> Option<Tick> {
    use crate::rules::{Effect, Query};
    fn bears(e: &Effect) -> bool {
        matches!(e, Effect::Birth { .. } | Effect::Bud { .. }) || e.branches().any(bears)
    }
    rules
        .processes
        .values()
        .filter(|p| p.commitments.iter().chain(&p.effects).any(bears))
        .filter(|p| {
            p.requires.iter().all(|q| match q {
                Query::Trait { who: crate::rules::Binding::Actor, key } => e.traits.contains(key),
                _ => true,
            })
        })
        .filter_map(|p| {
            p.requires.iter().filter_map(|q| match q {
                Query::Age { at_least } => Some(*at_least),
                _ => None,
            }).max()
        })
        .min()
}

impl Simulation {
    /// Whether the act `actor` was just visited for was one a policy chose,
    /// under a world that keeps learning.
    pub(crate) fn learns(&self, actor: Id) -> bool {
        rule(&self.genesis.rules).is_some() && self.chosen.learns(actor)
    }

    /// Whether an act left `actor` strictly better off: accepted, and its
    /// mood higher, or its mood the same and more matter held. Only this
    /// is credit to an act and to the one pending (an act that costs
    /// nothing and changes nothing serves at `served_by`, but earns none).
    pub(crate) fn bettered_by(&self, actor: Id, (mood, held): (i64, u128), accepted: bool) -> bool {
        let (now, has) = self.wellbeing(actor);
        accepted && (now > mood || (now == mood && has > held))
    }

    /// Whether an act served `actor`: it was accepted and left the critter
    /// better off in mood, or no worse in mood and no poorer in matter
    /// (`before` as `wellbeing` read it ahead of the act).
    pub(crate) fn served_by(&self, actor: Id, (mood, held): (i64, u128), accepted: bool) -> bool {
        let (now, has) = self.wellbeing(actor);
        accepted && (now > mood || (now == mood && has >= held))
    }

    /// `e`'s delta as it stands now: faded, a pending act whose horizon
    /// has passed settled as failed, and read only.
    pub(crate) fn learned_of(&self, e: &Entity) -> Option<Learned> {
        let l = rule(&self.genesis.rules)?;
        Some(l.settled(e.learned.as_ref()?, self.state.tick, |at| self.young_at(e, at))).filter(|k| !k.is_empty())
    }

    /// Whether `e` is younger than its lineage's maturity at `tick`.
    fn young_at(&self, e: &Entity, tick: Tick) -> bool {
        maturity(&self.genesis.rules, e).is_some_and(|m| tick.saturating_sub(e.born) < m)
    }

    /// What `actor`'s learned act weights add to the score of each act, by
    /// process, in the hand-written score's units: a weight is in 64ths of
    /// the pressure the world's strongest need gives at neutral mood, which
    /// is what a drive of one unit (64) makes of that need's term. So an act
    /// weighs as the needs it competes with do, and one that serves no
    /// need, as a move, can still be learned.
    pub(super) fn act_bonuses(&self, actor: Id) -> std::collections::BTreeMap<Key, i64> {
        let rules = &self.genesis.rules;
        let Some(learned) = self.body_at_start(actor).and_then(|e| self.learned_of(e)) else {
            return Default::default();
        };
        let unit = crate::meaning::needs(rules).iter().map(|n| -n.weight.min(0)).max().unwrap_or(0).max(1) * 100;
        let one = i64::from(super::policy::UNIT);
        learned.acts.into_iter().map(|(k, w)| (k, i64::from(w) * unit / one)).collect()
    }

    /// The needs the chosen act's terms scored, which learning moves.
    pub(super) fn pressed_by(&self, best: &Choice, mood: i64, pressing: &[&Need]) -> Vec<usize> {
        let rules = &self.genesis.rules;
        let Some(p) = best.process.as_ref().and_then(|id| rules.processes.get(id)) else {
            return vec![];
        };
        let serves = self.foraging_policy(&p.id).and_then(|f| rules.processes.get(&f.feed)).unwrap_or(p);
        super::choice::terms(rules, serves, p, mood, pressing, best.target)
            .needs
            .into_iter()
            .map(|(k, _)| k)
            .collect()
    }

    /// Moves `actor`'s delta by an act that `served` it or did not, on the
    /// needs that pressed when it chose, once the act is done. `stands` is
    /// how many members the act stood for: a cohort acting as one moves all
    /// its members alike and stays one row, a lone member is lifted into a
    /// row of its own only if its delta would change.
    pub(crate) fn learn_from(&mut self, actor: Id, stands: u64, act: &str, served: bool, bettered: bool) {
        let pressed = self.chosen.pressing(actor);
        self.learn_act(actor, &pressed, stands, Some((act, bettered)), served);
    }

    /// `learn_from` with the needs named and no act: a member's delta on
    /// each need, moved.
    #[cfg(test)]
    pub(crate) fn learn_step(&mut self, actor: Id, pressed: &[usize], stands: u64, served: bool) {
        self.learn_act(actor, pressed, stands, None, served);
    }

    /// A member's delta, moved on each need in `pressed` by whether the act
    /// `served` it, and on the kind of act, if one is named with whether it
    /// `bettered` the member: a better act steps its own weight up, and the
    /// pending act's with it, if one waits within the horizon (a lapsed one
    /// was settled as failed by `settled`). An act that did not better the
    /// member becomes the pending one where none waits. Where one waits it
    /// stays: the acts between a move and the meal it led to (an upkeep, a
    /// step on a journey) are no news of the move, and taking its place
    /// would hand the meal's credit to them. So the act pending is the
    /// first that did not serve, until it is credited or lapses; the later
    /// ones are dropped without credit or blame, their own needs' weights
    /// moved as always.
    fn learn_act(&mut self, actor: Id, pressed: &[usize], stands: u64, act: Option<(&str, bool)>, served: bool) {
        let Some(l) = rule(&self.genesis.rules) else { return };
        let tick = self.state.tick;
        let Some(e) = self.state.population.get(actor).filter(|e| e.alive) else { return };
        let was = e.learned.clone();
        let mut now = match &was {
            Some(k) => l.settled(k, tick, |at| self.young_at(e, at)),
            None => Learned { settled: tick, ..Default::default() },
        };
        let by = l.moved(self.young_at(e, tick));
        for &k in pressed {
            let w = now.weights.entry(k as u32).or_insert(0);
            *w = (*w + if served { by } else { -by }).clamp(-l.bound, l.bound);
        }
        now.weights.retain(|_, w| *w != 0);
        if let Some((act, bettered)) = act {
            let up = |now: &mut Learned, k: &str| {
                let w = now.acts.entry(k.into()).or_insert(0);
                *w = (*w + by).clamp(-l.bound, l.bound);
            };
            match (bettered, l.horizon) {
                (true, _) => {
                    up(&mut now, act);
                    if let Some(p) = now.pending.take().filter(|p| p.act != act) {
                        up(&mut now, &p.act);
                    }
                },
                (false, 0) => {},
                (false, _) if now.pending.is_none() => {
                    now.pending = Some(Pending { act: act.into(), since: tick });
                },
                (false, _) => {},
            }
        }
        let new = Some(now).filter(|k| !k.is_empty());
        if new != was {
            self.settle_learned(actor, stands, new);
        }
    }

    /// Writes `learned` for `actor` as an act's commit writes a body: the
    /// journal, the pass and the frame keep what they need of the group
    /// first, and the indexes are told after (see `stage::commit`).
    fn settle_learned(&mut self, actor: Id, stands: u64, learned: Option<Learned>) {
        let pop = &self.state.population;
        let Some((&base, group)) = pop.groups.range(..=actor).next_back() else { return };
        let (span, last, kept) = (group.count, base + group.count, Arc::clone(&group.entity));
        let alone = stands <= 1;
        let reached: Vec<Id> = match alone {
            true => vec![base, actor, if actor + 1 < last { actor + 1 } else { base }],
            false => vec![base],
        };
        if let (true, Some(pass)) = (alone, &mut self.pass) {
            pass.lifting(&self.state.population, actor);
        }
        if let Some(j) = &mut self.journal {
            j.groups(&self.state.population, &reached);
        }
        if let Some(frame) = &mut self.frame {
            match alone {
                true => frame.keep_body(actor, 1, kept),
                false => frame.keep_body(base, span, kept),
            }
        }
        let e = match alone {
            true => self.state.population.lift(actor),
            false => self.state.population.groups.get_mut(&base).map(|g| Ok(g.entity_mut())).unwrap_or(Err("unknown entity".into())),
        };
        match e {
            Ok(e) => e.learned = learned,
            Err(_) => return,
        }
        if let Some(t) = &mut self.targets {
            t.touch(&self.state.population, reached.iter().copied());
        }
        if let Some(j) = &mut self.journeys {
            j.touch(&self.state.population, reached.iter().copied());
        }
        if let Some(f) = &mut self.filed {
            let world = crate::schedule::World::of(&self.state, &self.genesis.rules);
            f.touch(&self.state.population, reached, &world);
        }
    }
}
