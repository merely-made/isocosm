// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bonds as individuals (ruling 554). The crowd keeps counts of states and
//! no identities, so whatever reads a relation runs member by member: a
//! parent and its unweaned young are lifted out of the counts as kin, each
//! with an identity and their relations kept, and act one by one. A process
//! that reads a relation runs on kin alone, each finding its target as the
//! core does: the first kin in identity order that the selector and the
//! requirements accept. Once no young of theirs is unweaned, and no parent
//! of theirs lives to nurse them, kin go back into the counts, their
//! relations dropped: nothing reads a relation once its young is weaned.

use super::{
    super::aggregate::{self, Pending},
    Crowd, Who, normalize,
};
use crate::{
    Result,
    rules::{Process, Query, Target},
    schema::*,
};
use std::collections::BTreeMap;

const CHILD: &str = "sim:child";
const PARENT: &str = "sim:parent";

/// Whether a process reads a relation, and so runs on kin alone.
pub(super) fn relational(p: &Process) -> bool {
    p.requires
        .iter()
        .any(|q| matches!(q, Query::Related { .. }))
}

fn accepts(selector: &Target, actor: &Entity, e: &Entity) -> bool {
    (!selector.same_place || e.place == actor.place)
        && selector.alive.is_none_or(|alive| alive == e.alive)
        && selector.lineage.as_ref().is_none_or(|l| *l == e.lineage)
        && (selector.among.is_empty() || selector.among.contains(&e.lineage))
}

impl Crowd<'_> {
    /// Lifts one member of the bin `e` out as kin; its identity.
    fn lift(&mut self, e: &Entity) -> Id {
        let slot = self.bins.get_mut(e).expect("a lifted member's bin exists");
        *slot -= 1;
        if *slot == 0 {
            self.bins.remove(e);
        }
        self.adopt(e.clone())
    }

    fn adopt(&mut self, e: Entity) -> Id {
        let id = self.next_kin;
        self.next_kin += 1;
        self.kin.insert(id, e);
        id
    }

    /// A birth's children, where they are born unweaned to a living parent:
    /// kin of their parent, both ways related, each knowing what weans it.
    /// `parent` is the bonded parent, lifted from its bin where it was not
    /// kin already.
    pub(super) fn bond(&mut self, parent: (&Who, &Entity), young: &Key, children: Vec<Entity>) {
        let p = match parent.0 {
            Who::Kin(id) => *id,
            Who::Bin => self.lift(parent.1),
        };
        for child in children {
            let c = self.adopt(child);
            self.relations.insert((p, CHILD.into(), c));
            self.relations.insert((c, PARENT.into(), p));
            self.young.insert(c, young.clone());
        }
    }

    /// Whether `born` bonds its children to their parent: they carry a
    /// trait their weaning takes away, and the parent lives.
    pub(super) fn bonding(born: &Pending, after: &Entity) -> Option<Key> {
        match born {
            Pending::Hatch {
                young: Some(key), ..
            } if after.alive => Some(key.clone()),
            _ => None,
        }
    }

    fn bonded(&self, id: Id) -> bool {
        let Some(e) = self.kin.get(&id).filter(|e| e.alive) else {
            return false;
        };
        let unweaned = |c: &Id| {
            let weans = self.young.get(c);
            self.kin
                .get(c)
                .is_some_and(|y| y.alive && weans.is_some_and(|k| y.traits.contains(k)))
        };
        let living = |p: &Id| self.kin.get(p).is_some_and(|p| p.alive);
        self.relations.iter().any(|(s, kind, o)| {
            *s == id
                && ((kind == CHILD && unweaned(o))
                    || (kind == PARENT && unweaned(&id) && living(o)))
        }) && e.alive
    }

    /// Returns to the counts every kin no longer in a bond.
    pub(super) fn release(&mut self) {
        let loose: Vec<Id> = self
            .kin
            .keys()
            .copied()
            .filter(|id| !self.bonded(*id))
            .collect();
        for id in loose {
            let e = self.kin.remove(&id).expect("listed above");
            *self.bins.entry(normalize(e)).or_default() += 1;
            self.relations.retain(|(s, _, o)| *s != id && *o != id);
            self.young.remove(&id);
        }
    }

    /// A process reading a relation, run on kin alone: each finds its
    /// target as the core does and acts on it; members in the counts hold no
    /// bonds, so theirs is blocked.
    pub(super) fn related(&mut self, p: &Process, acting: Vec<(Entity, u64, Who)>) -> Result<()> {
        let selector = p
            .target
            .as_ref()
            .ok_or(format!("{} relates to no target", p.id))?;
        let rules = &self.world.genesis.rules;
        let start = self.sites.clone();
        let draws = BTreeMap::new();
        for (e, n, who) in acting {
            self.work.evaluations += 1;
            self.work.represented += n;
            let Who::Kin(id) = who else {
                self.work.blocked += n;
                continue;
            };
            let a = aggregate::Act {
                start: &start[&e.place],
                count: 1,
                tick: self.tick,
                rules,
                lineages: Some(&self.lineages),
                draws: &draws,
                meal: None,
            };
            let chosen = self.kin.iter().find(|(c, t)| {
                let related = |kind: &Key| Ok(self.relations.contains(&(id, kind.clone(), **c)));
                **c != id
                    && accepts(selector, &e, t)
                    && aggregate::holds_on(p, (&e, t), &related, &a)
            });
            let Some((c, target)) = chosen.map(|(c, t)| (*c, t.clone())) else {
                self.work.blocked += 1;
                continue;
            };
            let related = |kind: &Key| Ok(self.relations.contains(&(id, kind.clone(), c)));
            let mut site = self.sites[&e.place].clone();
            let done = aggregate::act_on(p, (&e, &target), &related, &mut site, a)?;
            let Some((acted, target)) = done else {
                self.work.blocked += 1;
                continue;
            };
            if !acted.born.is_empty() {
                return Err(format!("{}: a related act's births run individually", p.id));
            }
            self.sites.insert(e.place, site);
            self.learn(&acted.lessons);
            self.kin.insert(id, acted.member);
            self.kin.insert(c, target);
            self.work.accepted += 1;
        }
        Ok(())
    }
}
