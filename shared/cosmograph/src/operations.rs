// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native declarations and finite, recorded game geometry resolutions.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::{Result, Session, Command, schema::*, simulation::{Simulation, Work}};
use isometer_space::{Edit, places::PlaceId};

pub mod geometry;
pub use geometry::{Geometry, Window};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change { Carve, Deposit { material: Key } }

/// The native action, independent of the game's chosen edit shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub action: Key,
    pub process: Key,
    pub tick: Tick,
    pub revision: Key,
    pub actor: Id,
    pub target: Option<Id>,
    pub site: Id,
    pub patch: PlaceId,
    pub body_revision: u64,
    pub terrain_revision: Key,
    pub change: Change,
    pub cells: u64,
    pub reach: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution { pub request: Request, pub geometry: Geometry }

/// A read-only native snapshot at the unresolved action, never a new owner.
#[derive(Clone, Debug)]
pub struct Pending { pub request: Request, pub sim: Box<Simulation> }

#[derive(Clone, Debug)]
pub(crate) struct StagedTerrain {
    pub request: Request,
    pub answered: bool,
    pub edit: Option<Edit>,
    pub gained: u128,
    pub paid: u128,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Resolutions {
    pub inputs: BTreeMap<Key, Resolution>,
    pub refused: BTreeMap<Key, (Request, String)>,
    pub allow_refusals: bool,
    pub used: BTreeSet<Key>,
    pub pending: Option<Pending>,
    pub work: Option<Work>,
}

impl Resolutions {
    pub(crate) fn of(inputs: &[Resolution]) -> Result<Self> {
        let mut held = Self::default();
        for input in inputs {
            if held.inputs.insert(input.request.action.clone(), input.clone()).is_some() {
                return Err("duplicate geometry resolution".into());
            }
        }
        Ok(held)
    }
}

pub enum Preparation { Pending(Pending), Ready(Prepared) }

/// Publication checks that the original native Session has not changed.
pub struct Prepared { candidate: Session, source: Key, pub work: Work, flows: Vec<crate::flows::Flow> }

pub(crate) fn source(session: &Session) -> Key {
    let _span = crate::profile::Span::new("prepared.source");
    crate::digest(&(&session.branch, &session.entries, session.sim.state_hash(),
        session.sim.genesis()))
}

impl Prepared {
    pub fn publish(self, session: &mut Session) -> Result<crate::flows::FlowResult<Work>> {
        if source(session) != self.source { return Err("geometry candidate is stale".into()); }
        let tick = self.candidate.sim.state().tick;
        *session = self.candidate;
        Ok(crate::flows::FlowResult { tick, result: self.work, flows: self.flows })
    }
}

impl Session {
    /// Repeated preflight starts from this same source; nothing is spent here.
    pub fn prepare_geometry(&self, ticks: Tick, inputs: &[Resolution]) -> Result<Preparation> {
        let mut candidate = self.clone();
        let command = Command::AdvanceGeometry { ticks, resolutions: inputs.to_vec() };
        match candidate.command_with_flows(command) {
            Ok(outcome) => {
                let work = candidate.sim.geometry_resolutions.work.take()
                    .ok_or("geometry advance omitted its native work")?;
                Ok(Preparation::Ready(Prepared { candidate, source: source(self), work, flows: outcome.flows }))
            },
            Err(why) => match candidate.sim.geometry_resolutions.pending.take() {
                Some(pending) => Ok(Preparation::Pending(pending)),
                None => Err(why),
            },
        }
    }

    pub(crate) fn advance_geometry(&mut self, ticks: Tick, inputs: &[Resolution]) -> Result<String> {
        if ticks != 1 { return Err("geometry advances one native tick at a time".into()); }
        if inputs.len() > self.sim.genesis().rules.limits.events_per_advance {
            return Err("geometry resolutions exceed the declared operation budget".into());
        }
        let mut candidate = self.clone();
        candidate.sim.geometry_resolutions = Resolutions::of(inputs)?;
        let work = match candidate.advance(ticks) {
            Ok(work) => work,
            Err(why) => {
                self.sim.geometry_resolutions.pending = candidate.sim.geometry_resolutions.pending.take();
                return Err(why);
            },
        };
        let supplied = &candidate.sim.geometry_resolutions;
        if supplied.used.len() != supplied.inputs.len() {
            return Err("geometry resolution does not name an executed native action".into());
        }
        candidate.sim.geometry_resolutions = Resolutions::default();
        candidate.sim.geometry_resolutions.work = Some(work);
        // Work's evaluation count varies by execution mode; accepted time does not.
        let outcome = format!("geometry advanced {ticks}");
        *self = candidate;
        Ok(outcome)
    }
}

impl Simulation {
    pub fn terrain_revision(&self) -> Key {
        let sites: Vec<_> = self.state.sites.iter().map(|(id, site)|
            (id, site.terrain_seed, &site.conditions, &site.routes)).collect();
        crate::digest(&(self.genesis.seed, &self.genesis.world, &self.genesis.rules.skeleton,
            sites, &self.state.edits))
    }

    pub(crate) fn capture_geometry(&mut self, terrain: &Option<StagedTerrain>) {
        let Some(terrain) = terrain else { return; };
        if terrain.answered {
            if self.geometry_resolutions.allow_refusals {
                self.geometry_resolutions.used.insert(terrain.request.action.clone());
            }
            return;
        }
        if self.geometry_resolutions.pending.is_some() { return; }
        let mut sim = self.clone();
        sim.clear_owned_capture();
        sim.geometry_resolutions = Resolutions::default();
        sim.journal = None;
        self.geometry_resolutions.pending = Some(Pending {
            request: terrain.request.clone(), sim: Box::new(sim),
        });
    }

    pub(crate) fn geometry_ready(&self) -> Result<()> {
        if self.geometry_resolutions.pending.is_some() || self.spatial_resolutions.pending.is_some()
            || self.spatial_resolutions.birth_pending.is_some() {
            return Err("native operation needs game geometry".into());
        }
        Ok(())
    }

    pub(crate) fn commit_geometry(&mut self, terrain: Option<StagedTerrain>) {
        let Some(terrain) = terrain else { return; };
        let edit = terrain.edit.expect("only resolved geometry can commit");
        self.conserved = self.conserved.checked_add(terrain.gained)
            .and_then(|n| n.checked_sub(terrain.paid)).expect("staged geometry pricing checked");
        self.state.edits.push(Edited { tick: self.state.tick, site: terrain.request.site,
            edit, by: Some(terrain.request.actor) });
        self.geometry_resolutions.used.insert(terrain.request.action);
    }
}

pub(crate) fn balanced(terrain: &Option<StagedTerrain>, before: u128, after: u128) -> bool {
    let Some(terrain) = terrain else { return before == after; };
    terrain.edit.is_some() && before.checked_add(terrain.gained)
        .and_then(|n| n.checked_sub(terrain.paid)) == Some(after)
}

pub(crate) fn validate(process: &crate::rules::Process, change: &Change,
    cells: &crate::rules::Amount, reach: u64) -> Result<()> {
    if process.causation != crate::rules::Causation::Choice
        || *cells == crate::rules::Amount::Fixed(0) || reach > i64::MAX as u64
    { return Err("terrain operations require a chosen act, cells and bounded reach".into()); }
    if let Change::Deposit { material } = change
        && (material.is_empty() || material == "world:air")
    { return Err("terrain deposition names no material".into()); }
    Ok(())
}

pub fn check_geometry(sim: &Simulation, request: &Request, geometry: &Geometry) -> Result<()> {
    geometry::price(sim, request, geometry).map(|_| ())
}
