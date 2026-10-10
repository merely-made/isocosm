// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A minimal native lineage revision (ruling 752), ahead of the lineages
//! family: a line may commit a variant of its development recipe drawn
//! from what it has learned, so 684's boundary has something to weigh. A
//! variant bears a learned kind its recipe does not yet name on one tagma.

use crate::{Result, Session, rules::Development, schema::*, simulation::Simulation};
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
pub fn revisions(session: &Session, lineage: &str) -> Vec<super::interim::Candidate> {
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
    variants(d, most)
        .into_iter()
        .filter(|(_, v)| sim.revisable(lineage, v).is_ok())
        .map(|(name, development)| super::interim::Candidate {
            name,
            commands: vec![crate::history::Command::Revise {
                lineage: lineage.into(),
                development,
            }],
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
