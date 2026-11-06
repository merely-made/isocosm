// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Work reported by successful native Section draws. Retained callbacks add
//! nothing; the shared counter survives the producer's GPU-source lifetime.

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use isometer::{ResidencyStats, lens::BrickDiagnostics};
use serde::Serialize;

use crate::section::BodyFrameStats;

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct BodyBytes {
    pub draws: u64,
    pub mesh_bytes: Option<u64>,
    pub instance_bytes: Option<u64>,
    pub frame_bytes: Option<u64>,
}

impl BodyBytes {
    fn for_measurement(measurement: &str) -> Self {
        let (mesh_bytes, instance_bytes, frame_bytes) = match measurement {
            "renderer upload counters" => (Some(0), Some(0), Some(0)),
            "mesh estimates; tenant frame uploads not exposed" => (Some(0), None, None),
            "implicit primitive and frame bytes" => (None, None, Some(0)),
            _ => (None, None, None),
        };
        Self {
            draws: 0,
            mesh_bytes,
            instance_bytes,
            frame_bytes,
        }
    }

    fn add(&mut self, stats: &BodyFrameStats) {
        self.draws = self.draws.saturating_add(1);
        for (total, bytes) in [
            (&mut self.mesh_bytes, stats.mesh_upload_bytes),
            (&mut self.instance_bytes, stats.instance_upload_bytes),
            (&mut self.frame_bytes, stats.frame_upload_bytes),
        ] {
            if let Some(total) = total {
                *total = total.saturating_add(bytes);
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct TerrainWork {
    pub draws: u64,
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub leased_atlas_bytes: u64,
    pub pointer_write_calls: u64,
    pub atlas_write_calls: u64,
    pub uniform_write_calls: u64,
    pub resource_creations: u64,
    pub bind_group_rebuilds: u64,
    pub trace_passes: u64,
    pub readback_bytes: u64,
    pub residency_changes: u64,
    pub loaded_bricks: u64,
    pub evicted_bricks: u64,
    pub refreshed_bricks: u64,
    pub map_rebuilds: u64,
}

impl TerrainWork {
    fn add(&mut self, stats: BrickDiagnostics) {
        self.draws = self.draws.saturating_add(1);
        macro_rules! add { ($($field:ident),* $(,)?) => {
            $(self.$field = self.$field.saturating_add(u64::from(stats.$field));)*
        }; }
        add!(
            cpu_prepare_us,
            brick_upload_bytes,
            uniform_upload_bytes,
            leased_atlas_bytes,
            pointer_write_calls,
            atlas_write_calls,
            uniform_write_calls,
            resource_creations,
            bind_group_rebuilds,
            trace_passes,
            readback_bytes
        );
    }

    fn residency(&mut self, changes: u64, stats: ResidencyStats) {
        if changes == 0 {
            return;
        }
        self.residency_changes = self.residency_changes.saturating_add(changes);
        self.loaded_bricks = self.loaded_bricks.saturating_add(stats.loaded as u64);
        self.evicted_bricks = self.evicted_bricks.saturating_add(stats.evicted as u64);
        self.refreshed_bricks = self.refreshed_bricks.saturating_add(stats.refreshed as u64);
        self.map_rebuilds = self
            .map_rebuilds
            .saturating_add(u64::from(stats.rebuilt.is_some()));
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProducerCosts {
    pub scope: &'static str,
    pub successful_draws: u64,
    pub mesh_builds: u64,
    pub drawn_parts: u64,
    /// A separate bucket preserves each renderer's measurement qualification.
    /// Null byte fields are unavailable, not measured zero.
    pub body_bytes_by_measurement: BTreeMap<&'static str, BodyBytes>,
    /// None until a successful draw actually supplies tracer diagnostics.
    pub terrain: Option<TerrainWork>,
    pub last_draw_populated: bool,
    pub unavailable: [&'static str; 5],
    pub mesquite_byte_scope: &'static str,
}

impl Default for ProducerCosts {
    fn default() -> Self {
        Self {
            scope: "successful Section draws submitted on the host device; no GPU-duration or presentation-completion claim",
            successful_draws: 0,
            mesh_builds: 0,
            drawn_parts: 0,
            body_bytes_by_measurement: BTreeMap::new(),
            terrain: None,
            last_draw_populated: false,
            unavailable: [
                "heap_allocations",
                "input_to_present_latency",
                "physical_display_deadlines",
                "gpu_duration",
                "complete_tenant_upload_bytes",
            ],
            mesquite_byte_scope: "sum of available body byte components only, including qualified mesh estimates; unavailable components are excluded, not measured zero",
        }
    }
}

impl ProducerCosts {
    pub(crate) fn mesh_bytes(&self) -> u64 {
        self.body_bytes_by_measurement
            .values()
            .filter_map(|work| work.mesh_bytes)
            .fold(0, u64::saturating_add)
    }
    pub(crate) fn instance_bytes(&self) -> u64 {
        self.body_bytes_by_measurement
            .values()
            .filter_map(|work| work.instance_bytes)
            .fold(0, u64::saturating_add)
    }
}

#[derive(Clone, Default)]
pub(super) struct Costs(Rc<RefCell<ProducerCosts>>);

impl Costs {
    pub(super) fn snapshot(&self) -> ProducerCosts {
        self.0.borrow().clone()
    }
    pub(super) fn totals(&self) -> (u64, u64, u64) {
        let costs = self.0.borrow();
        (
            costs.successful_draws,
            costs.mesh_bytes(),
            costs.instance_bytes(),
        )
    }
    pub(super) fn populated(&self) -> bool {
        self.0.borrow().last_draw_populated
    }

    pub(super) fn drawn(
        &self,
        body: &BodyFrameStats,
        terrain: Option<BrickDiagnostics>,
        residency_changes: u64,
        residency: ResidencyStats,
        populated: bool,
    ) {
        let mut totals = self.0.borrow_mut();
        totals.successful_draws = totals.successful_draws.saturating_add(1);
        totals.mesh_builds = totals.mesh_builds.saturating_add(body.mesh_builds as u64);
        totals.drawn_parts = totals.drawn_parts.saturating_add(body.draw_parts as u64);
        totals.last_draw_populated = populated;
        if body.draw_parts > 0
            || body.mesh_builds > 0
            || body.body_upload_measurement.is_some()
            || body.mesh_upload_bytes > 0
            || body.instance_upload_bytes > 0
            || body.frame_upload_bytes > 0
        {
            let measurement = body
                .body_upload_measurement
                .unwrap_or("renderer upload counters");
            totals
                .body_bytes_by_measurement
                .entry(measurement)
                .or_insert_with(|| BodyBytes::for_measurement(measurement))
                .add(body);
        }
        if let Some(terrain) = terrain {
            let work = totals.terrain.get_or_insert_with(TerrainWork::default);
            work.add(terrain);
            work.residency(residency_changes, residency);
        }
    }
}

#[cfg(test)]
mod tests;
