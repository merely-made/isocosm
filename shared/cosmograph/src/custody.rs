// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored item custody keeps native identity, stocks and route authority.

use crate::{Result, rules::*, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub use isometer_core::{Attachment, Yaw};

mod admission;
mod follow;
mod burden;
mod geometry;
pub use geometry::{MountPose, Bounds, mount_pose, item_bounds};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action { Carry, Drop, Equip, Unequip }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub relation: Key,
    pub capacity: Expr,
    pub carry: Key,
    pub drop: Key,
    pub equip: Key,
    pub unequip: Key,
    pub mounts: BTreeSet<Attachment>,
}

impl Provider {
    pub fn process(&self, action: Action) -> &str {
        match action { Action::Carry => &self.carry, Action::Drop => &self.drop,
            Action::Equip => &self.equip, Action::Unequip => &self.unequip }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalogue { pub providers: BTreeMap<Key, Provider> }

impl Catalogue {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        let mut kinds = BTreeSet::new();
        for (key, provider) in &self.providers {
            crate::validation::key(key)?;
            crate::validation::key(&provider.relation)?;
            if !rules.relations.contains(&provider.relation) || !kinds.insert(&provider.relation) {
                return Err("item custody needs a distinct declared relation per provider".into());
            }
            provider.capacity.validate()?;
            if provider.mounts.is_empty() {
                return Err("item custody needs explicit admitted attachment placements".into());
            }
            if provider.capacity.draws() || provider.capacity.reads().iter().any(|read|
                read.body() != Binding::Actor || matches!(read.reading, Reading::Kept { .. })) {
                return Err("item capacity must read only the actual carrier without draws or kept values".into());
            }
            for read in provider.capacity.reads() {
                match read.reading {
                    Reading::Account { key, .. } | Reading::Room { key, .. }
                        if !rules.accounts.contains_key(key) => return Err("item capacity reads an undeclared account".into()),
                    Reading::Span { function, .. } | Reading::Cells { function, .. }
                        | Reading::CellMass { function, .. } | Reading::Carried { function, .. }
                        if !rules.functions.contains_key(function) => return Err("item capacity reads an undeclared function".into()),
                    Reading::Carried { lands, .. } if lands.iter().any(|key| !rules.accounts.contains_key(key)) =>
                        return Err("item capacity carriage reads undeclared destinations".into()),
                    Reading::CellWeight { who, .. } if *who != Binding::Part =>
                        return Err("item capacity cell weight requires an actual part sum".into()),
                    _ => {},
                }
            }
            for action in [Action::Carry, Action::Drop, Action::Equip, Action::Unequip] {
                let key = provider.process(action);
                crate::validation::key(key)?;
                let process = rules.processes.get(key).ok_or("item custody process is absent")?;
                if process.causation != Causation::Choice || process.target.is_none() {
                    return Err("item custody requires an authored targeted Choice process".into());
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub participant: Id, pub actor: Id, pub item: Id, pub provider: Key,
    pub action: Action, pub cause: Option<Key>,
    pub selections: Vec<crate::projects::DebitSelection>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustodyActed {
    pub owned: crate::projects::OwnedActed,
    pub provider: Key, pub actor: Id, pub item: Id, pub carried: bool,
    pub matter: u128, pub burden: u128, pub capacity: u64,
}

impl Simulation {
    pub fn custody_provider(&self, key: &str) -> Result<&Provider> {
        self.genesis.rules.item_custody.as_ref().and_then(|catalogue| catalogue.providers.get(key))
            .ok_or_else(|| "item has no explicitly authored custody provider".into())
    }
}
