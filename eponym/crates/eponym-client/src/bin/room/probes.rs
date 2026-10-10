// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The room's headed probes: RG3b's scoped validation failure and RG3d's
//! rebuild of every device client, each writing its receipt.

use super::*;
use std::path::PathBuf;

pub(super) struct HeadedValidationProbe {
    pub(super) policy: PresentationPolicy,
    pub(super) injected_attempt: Option<u64>,
    pub(super) surface_acquire_attempts: Vec<u64>,
    pub(super) surface_present_attempts: Vec<u64>,
    pub(super) suppressed_attempts: Vec<u64>,
}

impl HeadedValidationProbe {
    pub(super) fn from_env() -> Option<Self> {
        let value = std::env::var("PAREDROS_RG3_HEADED_PROBE").ok()?;
        let policy = match value.to_ascii_lowercase().as_str() {
            "awaited" | "awaited-diagnostic" => PresentationPolicy::AwaitedDiagnostic,
            "optimistic" => PresentationPolicy::Optimistic,
            _ => panic!("PAREDROS_RG3_HEADED_PROBE must be awaited or optimistic, got {value:?}"),
        };
        Some(Self {
            policy,
            injected_attempt: None,
            surface_acquire_attempts: Vec::new(),
            surface_present_attempts: Vec::new(),
            suppressed_attempts: Vec::new(),
        })
    }

    pub(super) fn inject_once(&mut self, device: &wgpu::Device, attempt: u64) {
        if self.injected_attempt.is_some() {
            return;
        }
        self.injected_attempt = Some(attempt);
        let source = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RG3b headed validation source"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RG3b headed validation target"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("RG3b disposable invalid tenant encoder"),
        });
        encoder.copy_buffer_to_buffer(&source, 0, &target, 0, 8);
        drop(encoder.finish());
    }

    pub(super) fn record_suppressed(&mut self, attempt: u64) {
        self.suppressed_attempts.push(attempt);
    }

    pub(super) fn completed_record<'a>(
        &self,
        health: &'a FrameHealth,
    ) -> Option<&'a eponym_client::frame_health::ValidationRecord> {
        let injected = self.injected_attempt?;
        let validation = health
            .validations()
            .iter()
            .find(|record| record.frame == injected)?;
        let suppressed = self.suppressed_attempts.first().copied()?;
        let healthy_presented = self
            .surface_present_attempts
            .iter()
            .any(|attempt| *attempt > suppressed);
        let policy_contract = match self.policy {
            PresentationPolicy::AwaitedDiagnostic => suppressed == injected,
            PresentationPolicy::Optimistic => {
                self.surface_present_attempts.contains(&injected) && suppressed > injected
            },
        };
        let suppressed_never_acquired = self
            .suppressed_attempts
            .iter()
            .all(|attempt| !self.surface_acquire_attempts.contains(attempt));
        let suppressed_never_presented = self
            .suppressed_attempts
            .iter()
            .all(|attempt| !self.surface_present_attempts.contains(attempt));
        (healthy_presented
            && policy_contract
            && suppressed_never_acquired
            && suppressed_never_presented)
            .then_some(validation)
    }

    pub(super) fn write_receipt(&self, validation: &eponym_client::frame_health::ValidationRecord) {
        let (policy, path) = match self.policy {
            PresentationPolicy::AwaitedDiagnostic => ("awaited", RG3B_AWAITED_RECEIPT),
            PresentationPolicy::Optimistic => ("optimistic", RG3B_OPTIMISTIC_RECEIPT),
        };
        let injected = self.injected_attempt.expect("headed fault was injected");
        let receipt = format!(
            concat!(
                "{{\n",
                "  \"gate\": \"RG3b headed surface suppression\",\n",
                "  \"policy\": \"{}\",\n",
                "  \"injected_attempt\": {},\n",
                "  \"validation\": {{\n",
                "    \"tenant_name\": \"{}\",\n",
                "    \"producer_path\": \"{}\",\n",
                "    \"attempt\": {},\n",
                "    \"captured\": true\n",
                "  }},\n",
                "  \"surface_acquire_attempts\": {:?},\n",
                "  \"surface_present_attempts\": {:?},\n",
                "  \"suppressed_attempts\": {:?},\n",
                "  \"suppressed_attempts_never_acquired\": true,\n",
                "  \"suppressed_attempts_never_presented\": true,\n",
                "  \"healthy_frame_presented_after_suppression\": true,\n",
                "  \"shared_device\": true,\n",
                "  \"actual_surface_calls_observed\": true,\n",
                "  \"scope_limit\": \"native surface acquisition and presentation; no transactional rollback of renderer-internal bookkeeping\"\n",
                "}}\n"
            ),
            policy,
            injected,
            validation.tenant_name,
            validation.producer_path,
            validation.frame,
            self.surface_acquire_attempts,
            self.surface_present_attempts,
            self.suppressed_attempts,
        );
        let path = PathBuf::from(path);
        std::fs::create_dir_all(path.parent().expect("headed receipt directory"))
            .expect("create headed receipt directory");
        std::fs::write(&path, receipt).expect("write headed validation receipt");
        println!("RG3b headed {policy} receipt: {}", path.display());
    }
}

pub(super) struct RebuildProbe {
    pub(super) injected_attempt: Option<u64>,
    pub(super) fault_generation: Option<u64>,
    pub(super) rebuilt_generation: Option<u64>,
    pub(super) surface_acquire_attempts: Vec<u64>,
    pub(super) surface_present_attempts: Vec<u64>,
    pub(super) suppressed_attempts: Vec<u64>,
}

impl RebuildProbe {
    pub(super) fn from_env() -> Option<Self> {
        let value = std::env::var("PAREDROS_RG3_REBUILD_PROBE").ok()?;
        match value.to_ascii_lowercase().as_str() {
            "" | "0" | "false" => return None,
            "1" | "true" => {},
            _ => panic!("PAREDROS_RG3_REBUILD_PROBE must be 0/false or 1/true, got {value:?}"),
        }
        Some(Self {
            injected_attempt: None,
            fault_generation: None,
            rebuilt_generation: None,
            surface_acquire_attempts: Vec::new(),
            surface_present_attempts: Vec::new(),
            suppressed_attempts: Vec::new(),
        })
    }

    pub(super) fn inject_once(&mut self, live: &Live, attempt: u64) {
        if self.injected_attempt.is_some() {
            return;
        }
        self.injected_attempt = Some(attempt);
        self.fault_generation = Some(live.device_generation);
        live.health
            .lock()
            .expect("frame health lock")
            .latch_uncaptured_error("synthetic RG3d shared-device fault");
    }

    pub(super) fn record_rebuild(&mut self, fault_attempt: u64, rebuilt_generation: u64) {
        self.suppressed_attempts.push(fault_attempt);
        self.rebuilt_generation = Some(rebuilt_generation);
    }

    pub(super) fn complete(&self, current_generation: u64) -> bool {
        let Some(injected) = self.injected_attempt else {
            return false;
        };
        let Some(fault_generation) = self.fault_generation else {
            return false;
        };
        let Some(rebuilt_generation) = self.rebuilt_generation else {
            return false;
        };
        rebuilt_generation > fault_generation
            && current_generation == rebuilt_generation
            && self.suppressed_attempts == [injected]
            && !self.surface_acquire_attempts.contains(&injected)
            && !self.surface_present_attempts.contains(&injected)
            && self
                .surface_present_attempts
                .iter()
                .any(|attempt| *attempt > injected)
    }

    pub(super) fn write_receipt(&self) {
        let receipt = format!(
            concat!(
                "{{\n",
                "  \"gate\": \"RG3d shared-device rebuild-all\",\n",
                "  \"fault_injection\": \"synthetic host-latched uncaptured error\",\n",
                "  \"fault_attempt\": {},\n",
                "  \"fault_generation\": {},\n",
                "  \"rebuilt_generation\": {},\n",
                "  \"surface_acquire_attempts\": {:?},\n",
                "  \"surface_present_attempts\": {:?},\n",
                "  \"suppressed_attempts\": {:?},\n",
                "  \"fault_attempt_never_acquired\": true,\n",
                "  \"fault_attempt_never_presented\": true,\n",
                "  \"healthy_frame_presented_after_rebuild\": true,\n",
                "  \"preserved_host_resources\": [\"window\", \"surface\"],\n",
                "  \"recreated_shared_device_clients\": [\"adapter/device/queue\", \"body tenant\", \"optional DDA tenant\", \"netrender composer\", \"frame health and callbacks\"],\n",
                "  \"scope_limit\": \"proves the rebuild lifecycle after a synthetic shared fault; does not manufacture physical device loss\"\n",
                "}}\n"
            ),
            self.injected_attempt.expect("shared fault was injected"),
            self.fault_generation
                .expect("fault generation was recorded"),
            self.rebuilt_generation
                .expect("rebuilt generation was recorded"),
            self.surface_acquire_attempts,
            self.surface_present_attempts,
            self.suppressed_attempts,
        );
        let path = PathBuf::from(RG3D_REBUILD_RECEIPT);
        std::fs::create_dir_all(path.parent().expect("rebuild receipt directory"))
            .expect("create rebuild receipt directory");
        std::fs::write(&path, receipt).expect("write rebuild receipt");
        println!("RG3d headed rebuild-all receipt: {}", path.display());
    }
}
