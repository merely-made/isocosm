// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The deliberative methodology (ruling 683): a deliberative critter takes
//! one Choice process of those due, weighing its needs, its mood, its live
//! nudges and the bond each came by; its receipt's `foregone` lists the rest.
//! An answered nudge moves its bond by how the act served the critter (688).

use super::offers::{Elsewhere, served};
use super::{Aim, Answer, Nudge, Toward};
use crate::{
    meaning::{Named, Scene},
    rules::*,
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "choice/trace.rs"]
pub(crate) mod trace;

/// Each deliberative critter's choice at one tick, kept for an advance.
#[derive(Clone, Debug, Default)]
pub(crate) struct Chosen {
    tick: Tick,
    by: BTreeMap<Id, Choice>,
    /// What lineage policies chose, kept across advances until a reader
    /// takes it (M3.5).
    pub(crate) tally: BTreeMap<Key, super::policy::Tally>,
}

impl Chosen {
    /// Forgets an advance's choices, keeping the tally.
    pub(crate) fn clear(&mut self) {
        self.tick = 0;
        self.by.clear();
    }

    /// Whether `actor`'s choice this tick was a policy's, under a world that
    /// learns (M3.6).
    pub(crate) fn learns(&self, actor: Id) -> bool {
        self.by.get(&actor).is_some_and(|c| c.learns)
    }

    /// The needs, by place in the world's list, `actor`'s choice answered.
    pub(crate) fn pressing(&self, actor: Id) -> Vec<usize> {
        self.by.get(&actor).map(|c| c.pressed.clone()).unwrap_or_default()
    }

    /// An empty result lasts for one pass; a selected process lasts the tick (813).
    pub(crate) fn begin_pass(&mut self) {
        self.by.retain(|_, choice| choice.process.is_some());
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Choice {
    pub process: Option<Key>,
    pub target: Option<Id>,
    /// A Thing/Act nudge binds an identity, rather than an automatic prey draw.
    explicit_target: bool,
    /// Planning and visiting the feeding pass share one prey resolution (454).
    prey_resolved: bool,
    /// The live nudges, by index, that the chosen act answers.
    pub answers: Vec<usize>,
    /// What each considered process scored, and on what target.
    pub options: BTreeMap<Key, (i64, Option<Id>)>,
    /// Where a lineage policy chose: whether the hand-written score's best
    /// is above the chosen act's (M3.5).
    pub otherwise: Option<bool>,
    /// Where a lineage policy chose under a world that learns: the needs,
    /// by place in the world's list, the chosen act's terms scored (M3.6).
    pub pressed: Vec<usize>,
    /// Whether a policy chose it under a world that learns (M3.6).
    pub learns: bool,
    /// Whether it is a move toward what the body sensed there.
    pub led: bool,
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

/// What serves a need: its own account where it reads one of the actor's,
/// paid healing where it reads wounds (the 2026-10-18 note to ruling 227),
/// and otherwise any matter gained.
enum Serves<'a> {
    Account(&'a Key),
    Healing,
    Matter,
}

fn serves(q: &Query) -> Serves<'_> {
    if let Some(key) = crate::needs::account(q) {
        return Serves::Account(key);
    }
    match q {
        Query::Wounded { .. } => Serves::Healing,
        _ => Serves::Matter,
    }
}

/// Whether `e` serves a need: gives the actor its account, or any matter
/// where it names none, or heals where the need is a wound. A meal does not
/// heal, so eating alone never serves a wound. What the act hands on
/// (`passed`) is no matter gained: upkeep turns tissue to soil and drops it.
fn feeds(e: &Effect, need: &Serves, passed: &dyn Fn(&Key) -> bool) -> bool {
    let key = match need {
        Serves::Account(key) => Some(*key),
        Serves::Healing => {
            return match e {
                Effect::Heal { .. } => true,
                Effect::When { .. } => e.branches().any(|b| feeds(b, need, passed)),
                _ => false,
            };
        },
        Serves::Matter => None,
    };
    let is = |k: &Key| key.map_or_else(|| !passed(k), |key| key == k);
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
        Effect::When { .. } => e.branches().any(|b| feeds(b, need, passed)),
        _ => false,
    }
}

pub(super) fn serves_need(process: &Process, n: &Need) -> bool {
    let effects = || process.commitments.iter().chain(&process.effects);
    let passed = |k: &Key| {
        effects().any(|e| matches!(e, Effect::Transfer { from: Binding::Actor, account, .. } if account == k))
    };
    effects().any(|e| feeds(e, &serves(&n.query), &passed))
}

fn need_score(served: &[usize], mood: i64, pressing: &[&Need]) -> i64 {
    let needs: i64 = served.iter().map(|&k| -pressing[k].weight.min(0)).sum();
    needs * 100 * (100 + (-mood).max(0)) / 100
}

/// What a lineage policy reads of `process`, the pressing needs it serves
/// being `served`: each the hand-written score's own term, which sum to it
/// exactly.
pub(super) fn terms(rules: &Rules, served: &[usize], process: &Process, mood: i64, pressing: &[&Need],
    target: Option<Id>) -> super::policy::Terms {
    let all = crate::meaning::needs(rules);
    let lift = 100 + (-mood).max(0);
    let needs = served.iter().map(|&k| pressing[k]).filter_map(|n| {
        let k = all.iter().position(|m| std::ptr::eq(m, n))?;
        Some((k, -n.weight.min(0) * lift))
    }).collect();
    super::policy::Terms {
        needs,
        mood,
        priority: i64::from(process.priority),
        shape: [!moves(process).is_empty(), hunts(process), binds_place(process), target.is_some()],
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
            Effect::Condition { .. }
            | Effect::Spend { .. }
            | Effect::Grow { .. }
            | Effect::Heal { .. } => true,
            Effect::When { .. } => e.branches().any(at),
            _ => false,
        }
    }
    p.commitments.iter().chain(&p.effects).any(at)
}

pub(crate) fn hunts(p: &Process) -> bool {
    fn meal(e: &Effect) -> bool {
        matches!(
            e,
            Effect::Eat {
                from: Binding::Target,
                ..
            }
        ) || e.branches().any(meal)
    }
    p.commitments.iter().chain(&p.effects).any(meal)
}

/// The sites `p` moves its actor to.
pub(crate) fn moves(p: &Process) -> Vec<Id> {
    let to = |e: &Effect| match e {
        Effect::Move { destination, .. } | Effect::MovePaced { destination, .. } => Some(*destination),
        _ => None,
    };
    p.commitments
        .iter()
        .chain(&p.effects)
        .filter_map(to)
        .collect()
}

fn movement_only(p: &Process) -> bool {
    fn movement(effect: &Effect) -> bool {
        match effect {
            Effect::Move { .. } | Effect::MovePaced { .. } => true,
            Effect::When { .. } => {
                let branches: Vec<_> = effect.branches().collect();
                !branches.is_empty() && branches.into_iter().all(movement)
            }
            _ => false,
        }
    }
    let effects: Vec<_> = p.commitments.iter().chain(&p.effects).collect();
    !effects.is_empty() && effects.into_iter().all(movement)
}

impl Simulation {
    #[cfg(test)]
    pub(crate) fn recorded_choices(&self) -> (Tick, BTreeMap<Id, Choice>) {
        (self.chosen.tick, self.chosen.by.clone())
    }

    pub(crate) fn institution_chosen(&self,actor:Id,process:&str) -> bool {
        if self.pass.is_none() {return false;}
        match self.body_at_start(actor).map(|body|body.method) {
            Some(Method::Deliberative)=>self.chosen.tick==self.state.tick
                && self.chosen.by.get(&actor).is_some_and(|choice|choice.process.as_deref()==Some(process)),
            Some(Method::Inert)|None=>false,
            _=>true,
        }
    }
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
            self.chosen.clear();
            self.chosen.tick = tick;
        }
        // Empty results are invalidated only at a scheduled pass start (813),
        // so planning and visiting share one decision within that pass.
        if !self.chosen.by.contains_key(&actor) {
            let choice = self.choose(actor);
            #[cfg(test)]
            self.record_choice_attempt(actor, &process.id, &choice);
            if let Some(otherwise) = choice.otherwise {
                self.tally_choice(actor, otherwise, choice.led);
            }
            self.chosen.by.insert(actor, choice);
        }
        let c = &self.chosen.by[&actor];
        if c.process.as_deref() != Some(process.id.as_str()) {
            return Deliberated::Foregone;
        }
        // The methodology still chooses one process for the tick (683).
        // Automatic hunters draw against their feeding pass's start (454),
        // after earlier passes may have moved or killed the scored prey.
        if hunts(process) && !c.prey_resolved {
            let target = if c.explicit_target {
                c.target
            } else {
                self.choose_target(actor, process)
            };
            let place = self
                .body_at_start(actor)
                .map_or(super::PLACELESS, |e| e.place);
            let live = self.live_nudges(actor, tick);
            let mut answers = live.iter().copied()
                .filter(|&i| self.answers(&self.state.nudges[i], process, place, target))
                .collect::<Vec<_>>();
            super::Guidance::new(self, &live).score(self, process, place, &mut answers);
            let c = self
                .chosen
                .by
                .get_mut(&actor)
                .expect("choice kept for the tick");
            c.target = target;
            c.answers = answers;
            c.prey_resolved = true;
        }
        let c = &self.chosen.by[&actor];
        Deliberated::Chosen {
            target: c.target,
            answers: c.answers.clone(),
        }
    }

    /// The live mood of `actor`, read from the world's needs (ruling 227);
    /// none where the world keeps no mind or the actor is absent.
    pub fn mood(&self, actor: Id) -> Option<i64> {
        self.genesis.rules.mind.as_ref()?;
        self.state.population.get(actor)?;
        Some(self.minded(actor).0)
    }

    /// The live mood of `actor` and the weights of the needs pressing it.
    pub(crate) fn minded(&self, actor: Id) -> (i64, Vec<&Need>) {
        let e = self.state.population.get(actor);
        self.minded_in(e, e.and_then(|e| e.resident_site().and_then(|site| self.state.sites.get(&site))))
    }

    /// Scoring reads the same pass-start body and site as feasibility (454).
    /// Wellbeing keeps its live before/after readings for bond accounting.
    pub(super) fn choice_minded(&self, actor: Id) -> (i64, Vec<&Need>) {
        let e = self.body_at_start(actor);
        self.minded_in(e, e.and_then(|e| e.resident_site().and_then(|site| self.site_at_start(site))))
    }

    fn minded_in(&self, entity: Option<&Entity>, site: Option<&Site>) -> (i64, Vec<&Need>) {
        let Some(e) = entity else {
            return (0, vec![]);
        };
        let related = |_: &Key| -> crate::Result<bool> { Err("no target".into()) };
        let scene = Scene {
            actor: Some(e),
            target: Named::Unnamed,
            part: None,
            site,
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

    /// Whether an act of `p` at `place` on `target` answers `n`.
    fn answers(&self, n: &Nudge, p: &Process, place: Id, target: Option<Id>) -> bool {
        if n.aim.is_direction() || n.act.as_ref().is_some_and(|act| *act != p.id) {
            return false;
        }
        if n.aim == Aim::ActHere {
            return p.target.is_none() && n.toward == Toward::Site(place)
                && self.body_at_start(n.critter).and_then(Entity::resident_site) == Some(place);
        }
        let site = match n.toward {
            Toward::Thing(t) if n.aim == Aim::Act => return target == Some(t),
            Toward::Thing(t) => match self.body_at_start(t) {
                Some(e) => match e.resident_site() { Some(site) => site, None => return false },
                None => return false,
            },
            Toward::Site(s) => s,
        };
        let goes = moves(p);
        if crate::spread::directions(p).iter().any(|(_, destinations)| destinations.contains(&site)) {
            return true;
        }
        if self.body_at_start(n.critter).and_then(Entity::resident_site).is_none() { return false; }
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

    pub(crate) fn food_need_score(&self, actor: Id, process: &Process) -> i64 {
        let (mood, pressing) = self.choice_minded(actor);
        need_score(&served(process, &pressing, &Elsewhere::new()), mood, &pressing)
    }
    /// What `actor` weighs: the Choice processes due now, or all of them
    /// where `due_only` is false, as readings ask.
    pub(crate) fn consider(&self, actor: Id, due_only: bool) -> Choice {
        let tick = self.state.tick;
        let rules = &self.genesis.rules;
        let directing = rules.directing_ref();
        let Some(place) = self.body_at_start(actor).map(|e| e.place) else {
            return Choice::default();
        };
        let live = self.live_nudges(actor, tick);
        let (mood, pressing) = self.choice_minded(actor);
        let elsewhere = self.elsewhere(actor, place, &pressing);
        let (traits, standing) = (self.body_at_start(actor).map(|e| &e.traits), self.site_at_start(place));
        let guidance = super::Guidance::for_actor(self, actor, &live, Some(&pressing));
        let waiting = self.foraging_wait_score(actor);
        // A lineage policy scores in the hand-written score's place (M3.5).
        let policy = self.expressed(actor, directing.policy.expression);
        // What the member has learned of each kind of act scores it too (M3.6).
        let learnt = if policy.is_some() && directing.learning.is_some() { self.act_bonuses(actor) } else { Default::default() };
        let mut written = BTreeMap::new();
        let mut best = Choice::default();
        let mut best_served = Vec::new();
        let mut top = i64::MIN;
        let mut candidates = Vec::new();
        let born = self.body_at_start(actor).map_or(0, |e| e.born);
        let due = |p: &&Process| p.period.is_some() && self.due_for(p, born);
        let choices = rules
            .processes
            .values()
            .filter(|p| p.causation == Causation::Choice);
        // A trait the body lacks or a condition its site does not meet
        // refuses an act as its requirements would, before anything costly
        // is asked: most acts in a world are another lineage's or another
        // site's.
        let open = |p: &&Process| {
            p.requires.iter().all(|q| match q {
                Query::Trait { who: Binding::Actor, key } => traits.is_none_or(|t| t.contains(key)),
                Query::Condition { key, at_least } if key.starts_with("site:") => {
                    standing.is_none_or(|s| s.conditions.get(key).copied().unwrap_or(0) >= *at_least)
                },
                _ => true,
            })
        };
        for p in choices.filter(open).filter(|p| !due_only || due(p)) {
            if self.satisfied(actor, p) {
                continue;
            }
            let (nudged, target) = self.aim(actor, p, &live, place);
            if self.infeasible_guided(actor, target, p, place, Some(&guidance)).is_some() {
                continue;
            }
            let serves = self.foraging_policy(&p.id).and_then(|f| rules.processes.get(&f.feed)).unwrap_or(p);
            let offered = served(serves, &pressing, &elsewhere);
            let needs = need_score(&offered, mood, &pressing);
            let mut answers: Vec<usize> = live
                .iter()
                .copied()
                .filter(|&i| self.answers(&self.state.nudges[i], p, place, target))
                .collect();
            let ordinary_sway: i64 = answers
                .iter()
                .map(|&i| {
                    let n = &self.state.nudges[i];
                    let bond = self.bond(n.participant, actor).unwrap_or(0);
                    directing.sway * bond / directing.most
                })
                .sum();
            let direction = guidance.score(self, p, place, &mut answers);
            let sway = ordinary_sway + direction;
            let hand = needs + sway + i64::from(p.priority);
            let score = match &policy {
                Some(policy) => policy.score(&terms(rules, &offered, p, mood, &pressing, target)) + sway + learnt.get(&p.id).copied().unwrap_or(0),
                None => hand,
            };
            if policy.is_some() && !(waiting > 0 && movement_only(p) && hand < waiting) {
                written.insert(p.id.clone(), hand);
            }
            if waiting > 0 && movement_only(p) && score < waiting {
                continue;
            }
            best.options.insert(p.id.clone(), (score, target));
            // Wait if the cost of this direction outweighs the reasons to act.
            if direction < 0 && score < 0 { continue; }
            candidates.push((score, p.id.clone(), target, nudged.is_some(), answers.clone()));
            if score > top {
                top = score;
                best.process = Some(p.id.clone());
                best.target = target;
                best.explicit_target = nudged.is_some();
                best.answers = answers;
                best_served = offered;
            }
        }
        let ties: Vec<_> = candidates.iter().filter(|(score, _, _, _, _)| *score == top).collect();
        if due_only
            && directing.movement_ties == MovementTies::Seeded
            && ties.len() >= 2
            && ties.iter().all(|(_, id, _, _, _)| {
                rules.processes.get(id).is_some_and(movement_only)
            })
        {
            let tied_ids: Vec<_> = ties.iter().map(|(_, id, _, _, _)| id.clone()).collect();
            let set_fingerprint = crate::digest(&tied_ids);
            let set_key = u64::from_str_radix(&set_fingerprint[..16], 16)
                .expect("digest begins with hexadecimal digits");
            let draw = crate::draw(self.genesis.dynamics_seed(), "directing:movement-tie", &[actor, tick, set_key]);
            let (_, process, target, explicit_target, answers) = ties[(draw % ties.len() as u64) as usize];
            if best.process.as_ref() != Some(process) {
                best_served = rules.processes.get(process).map(|p| served(p, &pressing, &elsewhere)).unwrap_or_default();
            }
            best.process = Some(process.clone());
            best.target = *target;
            best.explicit_target = *explicit_target;
            best.answers = answers.clone();
        }
        if policy.is_some() && directing.learning.is_some() {
            // The needs the chosen act's terms scored, by place in the
            // world's list: what learning moves.
            let all = crate::meaning::needs(rules);
            best.pressed = best_served.iter().filter_map(|&k| all.iter().position(|m| std::ptr::eq(m, pressing[k]))).collect();
            best.learns = true;
        }
        if policy.is_some() {
            let most = written.values().copied().max();
            let chosen = best.process.as_ref().and_then(|p| written.get(p).copied());
            best.otherwise = Some(chosen.is_some_and(|c| most.is_some_and(|m| c < m)));
        }
        let gone = best.process.as_ref().and_then(|p| rules.processes.get(p)).map(moves).unwrap_or_default();
        best.led = gone.iter().any(|to| elsewhere.contains_key(to));
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
        let served = self.served_by(actor, (mood, held), accepted);
        let d = self.genesis.rules.directing();
        let site = self
            .state
            .population
            .get(actor)
            .and_then(Entity::resident_site).unwrap_or(super::PLACELESS);
        if let Some(j) = &mut self.journal {
            j.nudges(&self.state.nudges);
        }
        for &i in answers {
            if self.state.nudges[i].answer.is_some() { continue; }
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
