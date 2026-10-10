// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A minimal native lineage revision (ruling 752), ahead of the lineages
//! family: a line may commit a variant of its development recipe drawn
//! from what it has learned, so 684's boundary has something to weigh. A
//! variant bears a learned kind its recipe does not yet name on one tagma.
//! Since 765 a revision also declares a tract the line's members express
//! by acquisition, as Mesocosm's programs declared tracts, or folds in the
//! systems its members carry, 568's second door.

use crate::{
    Result, Session,
    rules::{Declared, Development, Seeding, System},
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

/// The variants of `d` a line could commit, named, at most `most`: each
/// learned kind its recipe lacks, borne on each tagma in turn.
pub fn variants(d: &Development, most: usize) -> Vec<(Key, Development)> {
    let named = d.recipe.kinds();
    let learned = d.lexicon.iter().filter(|k| !named.contains(*k));
    let mut out = Vec::new();
    for kind in learned {
        for i in 0..d.recipe.tagmata.len() {
            let mut v = d.clone();
            let t = &mut v.recipe.tagmata[i];
            t.bears = Some(kind.clone());
            t.per_segment = t.per_segment.max(1);
            out.push((format!("revision:{i}-bears-{kind}"), v));
        }
    }
    out.truncate(most);
    out
}

/// The boundary's candidates for `lineage`: its recipe's variants, each the
/// command that commits it, those the world would refuse left out.
pub fn revisions(session: &Session, lineage: &str) -> Vec<super::boundary::Candidate> {
    let sim = &session.sim;
    let most = sim.genesis().rules.directing().variants as usize;
    let Some(d) = sim
        .state()
        .lineages
        .get(lineage)
        .and_then(|l| l.development.as_ref())
    else {
        return vec![];
    };
    offered(session, lineage, d, most)
        .into_iter()
        .filter(|(_, v)| sim.revisable(lineage, v).is_ok())
        .map(|(name, development)| super::boundary::Candidate {
            name,
            commands: vec![crate::history::Command::Revise {
                lineage: lineage.into(),
                development,
            }],
        })
        .collect()
}

/// Everything a line could commit, named: its recipe's variants, the
/// tracts it could declare and the systems it could fold in, at most
/// `most` of each.
pub fn offered(
    session: &Session,
    lineage: &str,
    d: &Development,
    most: usize,
) -> Vec<(Key, Development)> {
    let mut out = variants(d, most);
    out.extend(declarations(session, lineage, d).into_iter().take(most));
    out.extend(folds(session, lineage, d).into_iter().take(most));
    out
}

/// The line's living members, each with how many it stands for.
fn members<'a>(session: &'a Session, lineage: &'a str) -> impl Iterator<Item = (&'a Entity, u64)> {
    let groups = session.sim.state().population.groups.values();
    groups
        .filter(move |g| g.entity.alive && g.entity.lineage == lineage)
        .map(|g| (&g.entity, g.count))
}

/// Tracts the line's members express by acquisition on a part of some
/// shape, which `d` does not yet declare: each a variant declaring it at
/// the most cells any member holds there.
fn declarations(session: &Session, lineage: &str, d: &Development) -> Vec<(Key, Development)> {
    let rules = &session.sim.genesis().rules;
    let acquired = |f: &Key| {
        rules
            .functions
            .get(f)
            .is_some_and(|x| x.seeding == Seeding::Acquired)
    };
    let mut found: BTreeMap<(Key, Key), u32> = BTreeMap::new();
    for (e, _) in members(session, lineage) {
        for (id, p) in e.living() {
            let Some(shape) = crate::anatomy::name(e, id) else {
                continue;
            };
            for (f, n) in p.cells.iter().filter(|(f, n)| **n > 0 && acquired(f)) {
                let slot = found.entry((shape.to_string(), f.clone())).or_default();
                *slot = (*slot).max(*n);
            }
        }
    }
    let declared = |shape: &Key, f: &Key| {
        d.tracts
            .iter()
            .any(|t| &t.shape == shape && &t.function == f)
    };
    found
        .into_iter()
        .filter(|((shape, f), _)| !declared(shape, f))
        .map(|((shape, function), cells)| {
            let mut v = d.clone();
            let name = format!("revision:declare-{function}-on-{shape}");
            v.tracts.push(Declared {
                shape,
                function,
                cells,
            });
            v.tracts.sort();
            (name, v)
        })
        .collect()
}

/// The systems the line's members carry, most members first, each a
/// variant folding them in, those it already folded left out.
fn folds(session: &Session, lineage: &str, d: &Development) -> Vec<(Key, Development)> {
    let mut held: Vec<(BTreeMap<Key, System>, u64)> = vec![];
    for (e, n) in members(session, lineage) {
        match held.iter_mut().find(|(s, _)| *s == e.systems) {
            Some(slot) => slot.1 += n,
            None => held.push((e.systems.clone(), n)),
        }
    }
    held.sort_by(|a, b| b.1.cmp(&a.1));
    held.into_iter()
        .filter(|(s, _)| !s.is_empty() && *s != d.systems)
        .enumerate()
        .map(|(i, (systems, _))| {
            let mut v = d.clone();
            v.systems = systems;
            (format!("revision:fold-systems-{i}"), v)
        })
        .collect()
}

impl Simulation {
    /// Whether `lineage` may develop from `d`: its own lexicon, unchanged,
    /// and a recipe the world admits.
    pub(crate) fn revisable(&self, lineage: &str, d: &Development) -> Result<Lineage> {
        let l = self.state.lineages.get(lineage).ok_or("unknown lineage")?;
        let had = l
            .development
            .as_ref()
            .ok_or("the lineage develops from nothing")?;
        if had.lexicon != d.lexicon {
            return Err("a revision draws only on what the lineage has learned".into());
        }
        let revised = Lineage {
            development: Some(d.clone()),
            revision: l.revision + 1,
            ..l.clone()
        };
        let one = BTreeMap::from([(lineage.to_string(), revised.clone())]);
        crate::validation::development(&self.genesis.rules, &one)?;
        Ok(revised)
    }

    /// Commits a revision: the lineage develops from `d` from now on.
    pub(crate) fn revise(&mut self, lineage: &str, d: &Development) -> Result<()> {
        let revised = self.revisable(lineage, d)?;
        self.state.lineages.insert(lineage.into(), revised);
        Ok(())
    }
}
