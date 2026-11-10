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
    /// Whether the act `actor` was just visited for was one a policy chose
    /// from pressing needs, under a world that keeps learning.
    pub(crate) fn learns(&self, actor: Id) -> bool {
        rule(&self.genesis.rules).is_some() && self.chosen.pressed(actor)
    }

    /// Whether an act served `actor`: it was accepted and left the critter
    /// better off in mood, or no worse in mood and no poorer in matter
    /// (`before` as `wellbeing` read it ahead of the act).
    pub(crate) fn served_by(&self, actor: Id, (mood, held): (i64, u128), accepted: bool) -> bool {
        let (now, has) = self.wellbeing(actor);
        accepted && (now > mood || (now == mood && has >= held))
    }

    /// `e`'s delta as it stands now: faded, and read only.
    pub(crate) fn learned_of(&self, e: &Entity) -> Option<Learned> {
        let l = rule(&self.genesis.rules)?;
        Some(l.faded(e.learned.as_ref()?, self.state.tick)).filter(|k| !k.weights.is_empty())
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
    pub(crate) fn learn_from(&mut self, actor: Id, stands: u64, served: bool) {
        let pressed = self.chosen.pressing(actor);
        self.learn_step(actor, &pressed, stands, served);
    }

    /// `learn_from` with the needs named: a member's delta, moved on each.
    pub(crate) fn learn_step(&mut self, actor: Id, pressed: &[usize], stands: u64, served: bool) {
        let Some(l) = rule(&self.genesis.rules) else { return };
        let tick = self.state.tick;
        let Some(e) = self.state.population.get(actor).filter(|e| e.alive) else { return };
        let young = maturity(&self.genesis.rules, e).is_some_and(|m| tick.saturating_sub(e.born) < m);
        let was = e.learned.clone();
        let mut now = match &was {
            Some(k) => l.faded(k, tick),
            None => Learned { settled: tick, weights: Default::default() },
        };
        let by = l.moved(young);
        for &k in pressed {
            let w = now.weights.entry(k as u32).or_insert(0);
            *w = (*w + if served { by } else { -by }).clamp(-l.bound, l.bound);
        }
        now.weights.retain(|_, w| *w != 0);
        let new = (!now.weights.is_empty()).then_some(now);
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
