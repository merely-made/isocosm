// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded opt-in wall timings. Nested spans overlap; never saved authority.
use std::{sync::{OnceLock, atomic::{AtomicUsize, Ordering}}, time::Instant};

pub(crate) struct Span<'a> { label: &'a str, started: Option<Instant> }
impl<'a> Span<'a> {
    pub(crate) fn new(label: &'a str) -> Self {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        let enabled = *ENABLED.get_or_init(|| std::env::var_os("ISOCOSM_PROFILE_ACTIVE")
            .is_some_and(|value| value == "1"));
        Self { label, started: enabled.then(Instant::now) }
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        if let Some(started) = self.started {
            if COUNT.fetch_add(1, Ordering::Relaxed) < 4096 {
                eprintln!("[isocosm-active] {} {}us", self.label, started.elapsed().as_micros());
            }
        }
    }
}
