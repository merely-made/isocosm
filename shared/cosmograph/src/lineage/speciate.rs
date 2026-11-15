// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Splitting a line is an act, naming (legacy `Lineages::fork` and
//! `Intent::Speciate`): one founder leaves its line for a new one, which
//! inherits the parent's record whole; its kin keep the old line.

use crate::{Result, Session, history::Command, schema::*, simulation::Simulation};

/// The key a line named `name` is founded under.
pub fn key(name: &str) -> Key {
    format!("lineage:{name}")
}

/// The tick `lineage` split off, `None` for a founding line.
pub fn speciated_at(session: &Session, lineage: &str) -> Option<Tick> {
    session.entries.iter().find_map(|e| match &e.command {
        Command::Speciate { name, .. } if key(name) == lineage => Some(e.tick),
        _ => None,
    })
}

impl Simulation {
    /// Founds the line `name` off `founder`'s, with `founder` its one member.
    pub(crate) fn speciate(&mut self, founder: Id, name: &str) -> Result<Key> {
        let made = key(name);
        if name.is_empty() || self.state.lineages.contains_key(&made) {
            return Err(format!("{made} cannot be founded"));
        }
        let e = self
            .state
            .population
            .get(founder)
            .ok_or("unknown founder")?;
        if !e.alive {
            return Err("only a living founder splits a line".into());
        }
        let parent = e.lineage.clone();
        let line = self
            .state
            .lineages
            .get(&parent)
            .filter(|l| l.kingdom != "kingdom:world")
            .ok_or("the founder's line cannot split")?;
        let mut fork = Lineage {
            parent: Some(parent),
            ..line.clone()
        };
        // A fork keeps a share of its parent's regard (case 1214).
        match self.genesis.rules.directing().regard {
            Some(r) => fork.regard.values_mut().for_each(|v| *v = *v * i64::from(r.heritable) / 1000),
            None => {
                fork.regard.clear();
                fork.ate.clear();
            },
        }
        if self.genesis.rules.inherited_variation != 0
            && let Some(development) = &mut fork.development
            && development.recipe.tagmata.len() == e.soma.len()
            && development.grammar.as_ref().is_some_and(|grammar| {
                grammar.rules.iter().all(|rule| matches!(rule, crate::rules::Rule::Repeat { .. }))
            })
        {
            for (region, &segments) in development.recipe.tagmata.iter_mut().zip(&e.soma) {
                region.segments = segments;
            }
            development.grammar = Some(crate::inherit::grammar(&development.recipe));
            crate::validation::development(
                &self.genesis.rules,
                &std::collections::BTreeMap::from([(made.clone(), fork.clone())]),
            )?;
        }
        self.state.population.lift(founder)?.lineage = made.clone();
        self.state.lineages.insert(made.clone(), fork);
        Ok(made)
    }
}
