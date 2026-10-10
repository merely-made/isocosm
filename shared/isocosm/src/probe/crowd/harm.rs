// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    harm,
    rules::{Binding, Effect, Target},
};

fn effects(p: &Process) -> impl Iterator<Item = &Effect> {
    p.commitments
        .iter()
        .chain(&p.effects)
        .chain(p.risk.iter().flat_map(|r| &r.effects))
        .flat_map(|e| std::iter::once(e).chain(e.branches()))
}

pub(super) fn harms(p: &Process) -> bool {
    effects(p).any(|e| matches!(e, Effect::Wound { .. } | Effect::Rot { .. }))
}

fn accepts(s: &Target, actor: &Entity, e: &Entity) -> bool {
    (!s.same_place || actor.place == e.place)
        && s.alive.is_none_or(|a| e.alive == a)
        && s.lineage.as_ref().is_none_or(|l| e.lineage == *l)
        && (s.among.is_empty() || s.among.contains(&e.lineage))
}

impl Crowd<'_> {
    pub(super) fn harms(&mut self, p: &Process, acting: Vec<(Entity, u64, Who)>) -> Result<()> {
        let rules = &self.world.genesis.rules;
        let selector = p.target.as_ref().ok_or("a harm pass without a target")?;
        if p.causation != Causation::Agentless
            || p.risk.is_some()
            || p.note
            || !selector.same_place
            || !selector.weighted
        {
            return Err(
                "the crowd harm pass requires an agentless local process with a weighted target and without risk or notes"
                    .into(),
            );
        }
        let mut counts = BTreeMap::<Id, u64>::new();
        for (actor, n, _) in &acting {
            *counts.entry(actor.place).or_default() += *n;
        }
        if counts.values().any(|n| *n > 1) {
            return Err("the crowd harm pass permits one actor per site".into());
        }
        let harmful: Vec<_> = effects(p)
            .filter(|e| matches!(e, Effect::Wound { .. } | Effect::Rot { .. }))
            .collect();
        if harmful.len() != 1
            || harmful.iter().any(|e| {
                !matches!(
                    e,
                    Effect::Wound {
                        who: Binding::Target,
                        ..
                    } | Effect::Rot {
                        who: Binding::Target,
                        ..
                    }
                )
            })
        {
            return Err("the crowd harm pass permits one target wound or rot per act".into());
        }
        let start = self.sites.clone();
        let domain = format!("probe-harm:{}", p.id);
        let mut stream = Stream::new(crate::draw(self.dynamics, &domain, &[self.tick]));
        let slots = aggregate::slots(p)?;
        for (actor, n, actor_who) in acting {
            for _ in 0..n {
                let mut draws: BTreeMap<_, _> = slots
                    .iter()
                    .map(|(slot, bound)| (*slot, stream.below(*bound)))
                    .collect();
                let a = aggregate::Act {
                    start: &start[&actor.place],
                    count: 1,
                    tick: self.tick,
                    rules,
                    lineages: Some(&self.lineages),
                    draws: &draws,
                    meal: None,
                };
                let kin = self.kin.iter().map(|(id, e)| (e.clone(), 1, Who::Kin(*id)));
                let bins = self.bins.iter().map(|(e, n)| (e.clone(), *n, Who::Bin));
                let mut candidates = vec![];
                let mut total = 0u128;
                for (e, count, who) in bins.chain(kin) {
                    if !accepts(selector, &actor, &e)
                        || !aggregate::holds_on(p, (&actor, &e), &|_| Ok(false), &a)
                    {
                        continue;
                    }
                    let held = crate::meaning::mass(&crate::anatomy::books(&e), rules);
                    let weight = match self.variant {
                        Variant::Unweighted => 1,
                        _ => held,
                    };
                    if held > 0 {
                        total += weight * u128::from(count);
                        candidates.push((e, count, who, weight));
                    }
                }
                if total == 0 {
                    self.work.blocked += 1;
                    continue;
                }
                let mut at = (u128::from(stream.below(u64::MAX)) * total) >> 64;
                let chosen = candidates
                    .into_iter()
                    .find_map(|(e, count, who, weight)| {
                        let mass = weight * u128::from(count);
                        if at < mass {
                            Some((e, who))
                        } else {
                            at -= mass;
                            None
                        }
                    })
                    .ok_or("the harm draw missed its population")?;
                let (target, who) = chosen;
                for e in effects(p) {
                    if let Effect::Wound {
                        who: Binding::Target,
                        slot,
                        ..
                    } = e
                    {
                        let total = harm::cells(&target);
                        if total == 0 {
                            return Err("a hazard targeted no living cells".into());
                        }
                        if draws.insert(*slot, stream.below(total)).is_some() {
                            return Err("a wound's part draw reuses another draw slot".into());
                        }
                    }
                }
                let a = aggregate::Act {
                    start: &start[&actor.place],
                    count: 1,
                    tick: self.tick,
                    rules,
                    lineages: Some(&self.lineages),
                    draws: &draws,
                    meal: None,
                };
                let mut site = self.sites[&actor.place].clone();
                let Some((acted, after)) =
                    aggregate::act_on(p, (&actor, &target), &|_| Ok(false), &mut site, a)?
                else {
                    self.work.blocked += 1;
                    continue;
                };
                self.sites.insert(actor.place, site);
                let log = self.meals.entry(p.id.clone()).or_default();
                log.count += 1;
                log.held += crate::meaning::mass(&crate::anatomy::books(&target), rules);
                match who {
                    Who::Bin => self.moved(&target, after, 1),
                    Who::Kin(id) => {
                        self.kin.insert(id, after);
                    },
                }
                match actor_who {
                    Who::Bin if acted.member != actor => self.moved(&actor, acted.member, 1),
                    Who::Kin(id) => {
                        self.kin.insert(id, acted.member);
                    },
                    _ => {},
                }
                for born in acted.born {
                    let aggregate::Pending::Fragment { child } = born else {
                        return Err("the harm pass made a different birth".into());
                    };
                    *self.bins.entry(normalize(child)).or_default() += 1;
                }
                self.work.evaluations += 1;
                self.work.represented += 1;
                self.work.accepted += 1;
            }
        }
        Ok(())
    }
}
