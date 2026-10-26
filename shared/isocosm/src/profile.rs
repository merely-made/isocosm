// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded opt-in wall timings. Nested spans overlap; never saved authority.
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

pub(crate) struct Span<'a> {
    label: &'a str,
    started: Option<Instant>,
}
pub(crate) fn counts(label: &str, values: &[(&str, usize)]) {
    if enabled() && COUNT.fetch_add(1, Ordering::Relaxed) < limit() {
        eprint!("[isocosm-count] {label}");
        for (key, value) in values {
            eprint!(" {key}={value}");
        }
        eprintln!();
    }
}
static COUNT: AtomicUsize = AtomicUsize::new(0);
fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("ISOCOSM_PROFILE_ACTIVE").is_some_and(|value| value == "1")
    })
}
impl<'a> Span<'a> {
    pub(crate) fn new(label: &'a str) -> Self {
        Self {
            label,
            started: enabled().then(Instant::now),
        }
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            if COUNT.fetch_add(1, Ordering::Relaxed) < limit() {
                eprintln!(
                    "[isocosm-active] {} {}us",
                    self.label,
                    started.elapsed().as_micros()
                );
            }
        }
    }
}

/// Log cap; `ISOCOSM_PROFILE_LIMIT` raises it for whole multi-tick runs.
fn limit() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        std::env::var("ISOCOSM_PROFILE_LIMIT")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(4096)
    })
}
