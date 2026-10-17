// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Opt-in bounded CPU wall timings; nested spans overlap.
use std::{sync::{OnceLock, atomic::{AtomicUsize, Ordering}}, time::Instant};

pub(crate) struct Span { label: &'static str, started: Option<Instant> }
impl Span {
    pub(crate) fn new(label: &'static str) -> Self {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        let enabled = *ENABLED.get_or_init(|| std::env::var_os("MESOCOSM_PROFILE_ACTIVE")
            .is_some_and(|value| value == "1"));
        Self { label, started: enabled.then(Instant::now) }
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        if let Some(started) = self.started {
            if COUNT.fetch_add(1, Ordering::Relaxed) < limit() {
                eprintln!("[mesocosm-active] {} {}us", self.label, started.elapsed().as_micros());
            }
        }
    }
}
pub(crate) fn measure<T>(label: &'static str, work: impl FnOnce() -> T) -> T {
    let _span = Span::new(label);
    work()
}

/// Log cap; `MESOCOSM_PROFILE_LIMIT` raises it for whole multi-tick runs.
fn limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| std::env::var("MESOCOSM_PROFILE_LIMIT").ok().and_then(|n| n.parse().ok()).unwrap_or(4096))
}
