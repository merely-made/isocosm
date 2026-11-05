// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Direct call scopes for candidate preparation, independent of publication.
//! Cumulative counters share the producer Costs lifetime across GPU retirement.

use serde::Serialize;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

static NEXT_COST_EPOCH: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PreparationCosts {
    /// Diagnostic lifetime only; no world or presentation identity is allocated.
    pub cost_epoch: Option<u64>,
    pub scope: &'static str,
    pub stages: BTreeMap<&'static str, StageTotal>,
    pub counters: BTreeMap<&'static str, u64>,
    pub counter_overflowed: bool,
    pub last_target_extent: Option<[u32; 2]>,
}
impl Default for PreparationCosts {
    fn default() -> Self {
        Self {
            cost_epoch: NEXT_COST_EPOCH
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
                .ok(),
            scope: "cumulative attempted producer preparation; draw_work includes CPU framing/body/terrain preparation, internal resources and GPU command encoding; explicit resource_setup excludes resources created inside draw; queue_submit includes command encoder finish and the host queue call, without waiting for completion; fresh accepted_query_capture copies immutable query inputs before host acceptance, retained_query_clone shares them; stages are direct scopes, without a complete CPU decomposition or GPU-duration claim",
            stages: BTreeMap::new(),
            counters: BTreeMap::new(),
            counter_overflowed: false,
            last_target_extent: None,
        }
    }
}
impl PreparationCosts {
    pub(super) fn record(&mut self, name: &'static str, span: Measured) {
        self.stages.entry(name).or_default().add(span);
    }
    pub(super) fn count(&mut self, name: &'static str, count: u64) {
        let total = self.counters.entry(name).or_default();
        *total = total.checked_add(count).unwrap_or_else(|| {
            self.counter_overflowed = true;
            u64::MAX
        });
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StageTotal {
    pub calls: u64,
    pub wall_ns: u64,
    /// Available only while every call in this bucket supplied a valid delta.
    pub thread_cpu_ns: Option<u64>,
    /// Sum of valid calls only; availability counts qualify interval deltas.
    pub observed_thread_cpu_ns: u64,
    pub thread_cpu_observed_calls: u64,
    pub thread_cpu_unavailable_calls: u64,
    pub thread_cpu_rejected_calls: u64,
    pub rejected_thread_cpu_ns: Option<u64>,
    pub overflowed: bool,
}
impl Default for StageTotal {
    fn default() -> Self {
        Self {
            calls: 0,
            wall_ns: 0,
            thread_cpu_ns: Some(0),
            observed_thread_cpu_ns: 0,
            thread_cpu_observed_calls: 0,
            thread_cpu_unavailable_calls: 0,
            thread_cpu_rejected_calls: 0,
            rejected_thread_cpu_ns: None,
            overflowed: false,
        }
    }
}
impl StageTotal {
    fn add(&mut self, span: Measured) {
        self.calls = self.calls.saturating_add(1);
        self.wall_ns = self.wall_ns.checked_add(span.wall_ns).unwrap_or_else(|| {
            self.overflowed = true;
            u64::MAX
        });
        match span.thread_cpu_ns {
            Some(cpu) if cpu <= span.wall_ns => {
                self.thread_cpu_observed_calls = self.thread_cpu_observed_calls.saturating_add(1);
                self.observed_thread_cpu_ns = self
                    .observed_thread_cpu_ns
                    .checked_add(cpu)
                    .unwrap_or_else(|| {
                        self.overflowed = true;
                        u64::MAX
                    });
                if let Some(total) = self.thread_cpu_ns {
                    self.thread_cpu_ns = total.checked_add(cpu);
                    self.overflowed |= self.thread_cpu_ns.is_none();
                }
            },
            Some(cpu) => {
                self.thread_cpu_rejected_calls = self.thread_cpu_rejected_calls.saturating_add(1);
                self.rejected_thread_cpu_ns =
                    self.rejected_thread_cpu_ns.unwrap_or(0).checked_add(cpu);
                self.overflowed |= self.rejected_thread_cpu_ns.is_none();
                self.thread_cpu_ns = None;
            },
            None => {
                self.thread_cpu_unavailable_calls =
                    self.thread_cpu_unavailable_calls.saturating_add(1);
                self.thread_cpu_ns = None;
            },
        }
    }
}

pub(super) struct Measured {
    pub wall_ns: u64,
    pub thread_cpu_ns: Option<u64>,
}
pub(super) struct Timer {
    wall: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    cpu: netrender::profiling::ThreadCpuStamp,
}
impl Timer {
    pub(super) fn start() -> Self {
        let wall = Instant::now();
        Self {
            wall,
            #[cfg(not(target_arch = "wasm32"))]
            cpu: netrender::profiling::ThreadCpuStamp::now(),
        }
    }
    pub(super) fn finish(self) -> Measured {
        #[cfg(not(target_arch = "wasm32"))]
        let cpu = netrender::profiling::ThreadCpuStamp::now().elapsed_since(self.cpu);
        #[cfg(target_arch = "wasm32")]
        let cpu: Option<std::time::Duration> = None;
        let wall = self.wall.elapsed();
        Measured {
            wall_ns: u64::try_from(wall.as_nanos()).unwrap_or(u64::MAX),
            thread_cpu_ns: cpu.and_then(|d| u64::try_from(d.as_nanos()).ok()),
        }
    }
}

#[cfg(test)]
mod tests;
