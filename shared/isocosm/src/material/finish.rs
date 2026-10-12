// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Recipe;
use crate::{
    Result,
    rules::{AccountKind, Rules},
    schema::{Entity, Id, Ledger, Method, Provenance},
    simulation::State,
};

impl Recipe {
    /// Prepare one native inert output from an exact completed construction
    /// ledger. This reads authority and returns a staged object only. The
    /// accepting project stage must transfer the construction ledger into
    /// this output once, clear its former holder, and admit population/flows
    /// together. Neither `issued` nor the conserved total may increase.
    pub fn output(
        &self,
        rules: &Rules,
        state: &State,
        maker: Id,
        site: Id,
        construction: &Ledger,
    ) -> Result<Entity> {
        self.validate(rules)?;
        self.validate_lineages(&state.lineages)?;
        if construction != &self.stocks {
            return Err("material construction differs from its exact recipe stocks".into());
        }
        if state.population.get(maker).is_none() {
            return Err("material output has no native maker".into());
        }
        if !state.sites.contains_key(&site) {
            return Err("material output has no native site".into());
        }
        for key in self.stocks.keys() {
            let Some(AccountKind::Matter { lineage, .. }) = rules.accounts.get(key) else {
                return Err("material construction holds undeclared matter".into());
            };
            if !state.lineages.contains_key(lineage) {
                return Err(format!("material stock {key} has no native source lineage"));
            }
        }
        let lineage = &state.lineages[&self.lineage];
        Ok(Entity {
            lineage: self.lineage.clone(),
            kingdom: lineage.kingdom.clone(),
            scale: self.scale.clone(),
            provenance: Provenance::Made(maker),
            method: Method::Inert,
            place: site,
            arrived: state.tick,
            visits: vec![],
            journey: None,
            born: state.tick,
            alive: true,
            body_revision: 1,
            body: None,
            parts: Default::default(),
            traits: Default::default(),
            accounts: construction.clone(),
            skills: Default::default(),
            tenets: Default::default(),
            disposition: [0; 5],
            soma: vec![],
            systems: Default::default(),
            varied: vec![],
            patch: None,
            authored: None,
        })
    }
}
