// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use super::advance::StagedAdvance;
use crate::{Session, Command, flows::FlowResult};

/// An advisory refusal for parents absent from the product's physical source.
/// New successful arrivals and births still go to the ordinary provider.
#[derive(Clone, Debug)]
pub struct BirthFallback { pub known_parents: BTreeSet<Id>, pub refusal: String }

impl BirthFallback {
    pub(super) fn answer(&self, request: &BirthRequest, held: &Resolutions) -> Option<BirthResolution> {
        let parent = request.parent;
        if self.known_parents.contains(&parent)
            || held.reports.iter().any(|r| r.actor == parent && r.patch.is_ok())
            || held.birth_reports.iter().any(|r| r.request.child == parent && r.patch.is_ok()) {
            return None;
        }
        Some(BirthResolution { request: request.clone(), geometry: Err(self.refusal.clone()) })
    }
}

impl Session {
    /// Batches only the refusal prefix before the next ordinary birth question.
    /// Return receipts on pending attempts too: later answers must retain the
    /// same request snapshots and callback order as individual retries.
    pub fn try_advance_birth_spatial_with_fallback(&mut self, ticks: Tick,
        terrain: &[TerrainResolution], steps: &[Resolution], arrivals: &[ArrivalResolution],
        births: &[BirthResolution], fallback: &BirthFallback)
        -> Result<(SpatialAdvance, Vec<BirthResolution>)> {
        let command = Command::AdvanceBirthSpatial {
            ticks, terrain: terrain.to_vec(), steps: steps.to_vec(), arrivals: arrivals.to_vec(), births: births.to_vec(),
        };
        let (attempt, automatic) = self.stage_spatial_with(command, Some(fallback))?;
        let out = match attempt {
            StagedAdvance::Terrain(pending) => SpatialAdvance::Terrain(pending),
            StagedAdvance::Step(pending) => SpatialAdvance::Step(pending),
            StagedAdvance::Birth(pending) => SpatialAdvance::Birth(pending),
            StagedAdvance::Ready { candidate, work, flows } => {
                if automatic.is_empty() {
                    let tick = candidate.sim.state().tick;
                    *self = candidate;
                    return Ok((SpatialAdvance::Advanced(FlowResult { tick, result: work, flows }), automatic));
                }
                // No discovery state publishes: the same authenticated command
                // as individual callbacks, preserving their exact answer order.
                let mut expanded = births.to_vec();
                expanded.extend_from_slice(&automatic);
                self.try_advance_birth_spatial_with_flows(ticks, terrain, steps, arrivals, &expanded)?
            },
        };
        Ok((out, automatic))
    }
}
