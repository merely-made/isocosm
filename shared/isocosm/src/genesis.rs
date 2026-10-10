// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::AccountKind, simulation::Genesis};
use std::collections::BTreeSet;

impl Genesis {
    pub fn validate(&self) -> Result<()> {
        if self.version != crate::VERSION {
            return Err("unsupported world version".into());
        }
        self.rules.validate()?;
        self.population.validate(self.rules.limits.entities)?;
        if self.sites.is_empty()
            || self.sites.len() > self.rules.limits.sites
            || self.world.base_unit_micrometres == 0
        {
            return Err("invalid world extent".into());
        }
        wing_glyphs::Canon::new(self.world.canon.clone())?;
        for kind in self.rules.accounts.values() {
            if let AccountKind::Matter { lineage, .. } = kind
                && !self.lineages.contains_key(lineage)
            {
                return Err("matter has an absent provenance lineage".into());
            }
        }
        for (key, lineage) in &self.lineages {
            crate::validation::key(key)?;
            let mut seen = BTreeSet::from([key]);
            let mut parent = lineage.parent.as_ref();
            while let Some(p) = parent {
                if !seen.insert(p) {
                    return Err("cyclic lineage".into());
                }
                parent = self
                    .lineages
                    .get(p)
                    .ok_or("absent parent lineage")?
                    .parent
                    .as_ref();
            }
            if lineage
                .traits
                .iter()
                .any(|t| !self.rules.traits.contains(t))
            {
                return Err("unknown lineage trait".into());
            }
        }
        crate::validation::conversions(&self.rules, &self.lineages)?;
        crate::validation::development(&self.rules, &self.lineages)?;
        for process in self.rules.processes.values() {
            if process.target.as_ref().is_some_and(|t| {
                (t.lineage.iter().chain(&t.among)).any(|l| !self.lineages.contains_key(l))
            }) {
                return Err("target selector names an absent lineage".into());
            }
        }
        for (id, site) in &self.sites {
            for route in &site.routes {
                if !self.sites.contains_key(&route.to)
                    || route.to == *id
                    || route.travel == 0
                    || route.transmission > 1_000_000
                {
                    return Err("invalid site route".into());
                }
            }
            for condition in site.conditions.keys() {
                if !self.rules.conditions.contains(condition) {
                    return Err("unknown site condition".into());
                }
            }
            for account in site.accounts.keys() {
                if !self.rules.accounts.contains_key(account) {
                    return Err("unknown site account".into());
                }
            }
        }
        for group in self.population.groups.values() {
            let e = &group.entity;
            if e.born != 0 || e.arrived != 0 || !e.visits.is_empty() {
                return Err("founding bodies must begin at tick zero".into());
            }
            if !self.sites.contains_key(&e.place) || !self.lineages.contains_key(&e.lineage) {
                return Err("entity references an absent place or lineage".into());
            }
            for key in e.accounts.keys() {
                if !self.rules.accounts.contains_key(key) {
                    return Err("unknown entity account".into());
                }
            }
            for key in &e.traits {
                if !self.rules.traits.contains(key) {
                    return Err("unknown entity trait".into());
                }
            }
            for (&id, part) in &e.parts {
                if part.traits.iter().any(|t| !self.rules.traits.contains(t)) {
                    return Err("unknown part trait".into());
                }
                crate::validation::part(&self.rules, e, id)?;
                // Laid-out cells agree with the counts (766).
                if !part.tracts.is_empty() {
                    crate::mosaic::agrees(part, e.extent(id))?;
                }
                // A body's own matter lives in its parts (ruling 504).
                if e.bodied(id) {
                    let own = e.accounts.keys().find(|k| {
                        matches!(self.rules.accounts.get(*k),
                            Some(AccountKind::Matter { lineage, .. }) if *lineage == e.lineage)
                    });
                    if let Some(k) = own.filter(|k| e.accounts[*k] > 0) {
                        return Err(format!("a body with parts holds {k} outside them"));
                    }
                }
                // A declared name cannot lie (ruling 494): a bodied part
                // names a hollow or what its box and the tree read.
                let declared = e.declared(id);
                if e.bodied(id) && !declared.is_empty() {
                    let read = crate::anatomy::name(e, id);
                    if read != Some(declared) {
                        return Err(format!("a part declared {declared} reads as {read:?}"));
                    }
                }
                // Its geometry is the document's (674), which isometer keeps
                // acyclic; a part keyed outside it has none to read.
                if e.body.as_ref().is_some_and(|d| d.part(id).is_none()) {
                    return Err("a part outside its body's document".into());
                }
                if e.body.as_ref().is_some_and(|d| d.depth(id).is_none()) {
                    return Err("cyclic anatomy".into());
                }
            }
        }
        crate::map::validate(self)
    }
}
